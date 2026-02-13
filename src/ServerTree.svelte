<script lang="ts">
  import {
    getServerConnections,
    addServerConnection,
    removeServerConnection,
    connectToServer,
    disconnectFromServer,
    listTopics,
    type ServerConnectionSettings,
    type ServerConnectionRequest
  } from './lib/api';
  import {
    getFavorites,
    addFavoriteServer,
    addFavoriteTopic,
    removeFavorite,
    isFavorited,
    getFavoriteId,
    type FavoriteItem
  } from './lib/favorites';
  import {
    getHistoryGrouped,
    addHistoryServer,
    addHistoryTopic,
    clearHistory,
    formatHistoryTime,
    type HistoryItem
  } from './lib/history';

  // Exportable props for two-way binding
  export let selectedServerId: number | null = null;
  export let selectedTopic: string | null = null;

  // Component state
  let servers = new Map<number, ServerConnectionSettings>();
  let expandedServers = new Set<number>();
  let serverTopics = new Map<number, string[]>();
  let serverStates = new Map<number, 'disconnected' | 'connecting' | 'connected' | 'error'>();
  let favorites: FavoriteItem[] = [];
  let history: { [key: string]: HistoryItem[] } = { today: [], yesterday: [], thisWeek: [], older: [] };

  // UI state
  let showFavorites = false;
  let showHistory = false;
  let showAddServerDialog = false;

  // Add Server form state
  let newServerName = '';
  let newBootstrapServers = '';
  let newSecurityType = 'PLAINTEXT';
  let newZookeeperHosts = '';
  let newZookeeperChroot = '';

  // Load servers on component mount
  async function fetchServers() {
    try {
      const serverList = await getServerConnections();
      servers.clear();
      for (const server of serverList) {
        servers.set(server.id, server);
      }
    } catch (error) {
      console.error('Failed to load servers:', error);
    }
  }

  // Load favorites on component mount
  function loadFavorites() {
    favorites = getFavorites();
  }

  // Load history on component mount
  function loadHistory() {
    history = getHistoryGrouped();
  }

  // Connect to a server
  async function handleConnect(serverId: number) {
    serverStates.set(serverId, 'connecting');
    try {
      await connectToServer(serverId);
      serverStates.set(serverId, 'connected');
      // Auto-expand and load topics after successful connection
      expandedServers.add(serverId);
      await loadTopics(serverId);
    } catch (error) {
      console.error('Connection failed:', error);
      serverStates.set(serverId, 'error');
    }
  }

  // Disconnect from a server
  async function handleDisconnect(serverId: number) {
    try {
      await disconnectFromServer(serverId);
      serverStates.set(serverId, 'disconnected');
      serverTopics.delete(serverId);
      expandedServers.delete(serverId);
    } catch (error) {
      console.error('Disconnect failed:', error);
    }
  }

  // Toggle server expansion
  async function toggleServerExpansion(serverId: number) {
    if (expandedServers.has(serverId)) {
      expandedServers.delete(serverId);
    } else {
      expandedServers.add(serverId);
      const state = serverStates.get(serverId);
      if (state === 'connected') {
        await loadTopics(serverId);
      }
    }
  }

  // Load topics for a server
  async function loadTopics(serverId: number) {
    try {
      const topics = await listTopics(serverId);
      serverTopics.set(serverId, topics);
    } catch (error) {
      console.error('Failed to load topics:', error);
    }
  }

  // Select a server
  function selectServer(serverId: number) {
    selectedServerId = serverId;
    selectedTopic = null;

    // Add to history
    const server = servers.get(serverId);
    if (server) {
      addHistoryServer(serverId, server.name);
      loadHistory();
    }
  }

  // Select a topic
  function selectTopic(topic: string) {
    selectedTopic = topic;

    // Add to history
    const serverId = selectedServerId;
    const server = serverId ? servers.get(serverId) : null;
    if (server && topic && serverId !== null) {
      addHistoryTopic(serverId, server.name, topic);
      loadHistory();
    }
  }

  // Toggle favorite for server
  function toggleFavoriteServer(server: ServerConnectionSettings) {
    const favId = getFavoriteId('server', server.id);
    if (favId) {
      removeFavorite(favId);
    } else {
      addFavoriteServer(server.id, server.name);
    }
    loadFavorites();
  }

  // Toggle favorite for topic
  function toggleFavoriteTopic(serverId: number, serverName: string, topicName: string) {
    const favId = getFavoriteId('topic', serverId, topicName);
    if (favId) {
      removeFavorite(favId);
    } else {
      addFavoriteTopic(serverId, serverName, topicName);
    }
    loadFavorites();
  }

  // Check if server is favorited
  function isServerFavorited(serverId: number): boolean {
    return isFavorited('server', serverId);
  }

  // Check if topic is favorited
  function isTopicFavorited(serverId: number, topicName: string): boolean {
    return isFavorited('topic', serverId, topicName);
  }

  // Navigate to favorite item
  function navigateToFavorite(favorite: FavoriteItem) {
    if (favorite.type === 'server') {
      selectServer(favorite.serverId!);
      expandedServers.add(favorite.serverId!);
    } else if (favorite.type === 'topic') {
      selectServer(favorite.serverId!);
      expandedServers.add(favorite.serverId!);
      selectedTopic = favorite.topicName ?? null;
    }
  }

  // Navigate to history item
  function navigateToHistoryItem(item: HistoryItem) {
    if (item.type === "server") {
      selectServer(item.serverId!);
      expandedServers.add(item.serverId!);
    } else if (item.type === "topic") {
      selectServer(item.serverId!);
      expandedServers.add(item.serverId!);
      selectedTopic = item.topicName!;
    }
  }

  // Add Server dialog handlers
  function openAddServerDialog() {
    newServerName = '';
    newBootstrapServers = '';
    newSecurityType = 'PLAINTEXT';
    newZookeeperHosts = '';
    newZookeeperChroot = '';
    showAddServerDialog = true;
  }

  function closeAddServerDialog() {
    showAddServerDialog = false;
  }

  async function handleAddServer() {
    if (!newServerName.trim() || !newBootstrapServers.trim()) {
      alert('Please enter server name and bootstrap servers');
      return;
    }

    try {
      const request: ServerConnectionRequest = {
        name: newServerName.trim(),
        bootstrapServers: newBootstrapServers.trim(),
        securityType: newSecurityType,
        zookeeperHosts: newZookeeperHosts.trim() || undefined,
        zookeeperChroot: newZookeeperChroot.trim() || undefined,
      };

      const serverId = await addServerConnection(request);
      await fetchServers();
      closeAddServerDialog();

      // Auto-connect to the new server
      await handleConnect(serverId);
    } catch (error) {
      console.error('Failed to add server:', error);
      alert('Failed to add server: ' + error);
    }
  }

  async function handleDeleteServer(serverId: number, serverName: string) {
    if (!confirm(`Are you sure you want to delete server "${serverName}"?`)) {
      return;
    }

    try {
      await removeServerConnection(serverId);
      await fetchServers();

      // Clear selection if deleted server was selected
      if (selectedServerId === serverId) {
        selectedServerId = null;
        selectedTopic = null;
      }
    } catch (error) {
      console.error('Failed to delete server:', error);
      alert('Failed to delete server: ' + error);
    }
  }

  // Initialize
  fetchServers();
  loadFavorites();
  loadHistory();
</script>

<div class="server-tree">
  <div class="toolbar">
    <button class="btn-primary" on:click={openAddServerDialog}>
      <span>+</span>
      <span>Add Server</span>
    </button>
    <button class="btn-secondary" on:click={fetchServers}>
      <span>Refresh</span>
    </button>
    <button
      class="btn-secondary"
      on:click={() => { showFavorites = !showFavorites; showHistory = false; }}
      class:active={showFavorites}
    >
      <span class="fav-icon">⭐</span>
      <span>Favorites</span>
      {#if favorites.length > 0}
        <span class="fav-count">{favorites.length}</span>
      {/if}
    </button>
    <button
      class="btn-secondary"
      on:click={() => { showHistory = !showHistory; showFavorites = false; }}
      class:active={showHistory}
    >
      <span class="history-icon">🕒</span>
      <span>History</span>
    </button>
  </div>

  {#if showFavorites}
    <div class="favorites-section">
      <div class="favorites-header">
        <h3>⭐ Favorites</h3>
        <button class="btn-close" on:click={() => showFavorites = false}>×</button>
      </div>

      {#if favorites.length === 0}
        <div class="empty-favorites">
          <p>No favorites yet</p>
          <p class="hint">Click the star icon to add servers or topics to favorites</p>
        </div>
      {:else}
        <div class="favorites-list">
          {#each favorites as favorite}
            <div
              class="favorite-item"
              on:click={() => navigateToFavorite(favorite)}
              role="button"
              tabindex="0"
            >
              <span class="fav-type-icon">
                {favorite.type === 'server' ? '🖥' : '📄'}
              </span>
              <div class="fav-info">
                <span class="fav-name">
                  {favorite.type === 'server' ? favorite.serverName : favorite.topicName}
                </span>
                {#if favorite.type === 'topic'}
                  <span class="fav-server-name">{favorite.serverName}</span>
                {/if}
              </div>
              <button
                class="btn-remove-fav"
                on:click|stopPropagation={() => removeFavorite(favorite.id)}
                title="Remove from favorites"
              >
                ×
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if showHistory}
    <div class="history-section">
      <div class="history-header">
        <h3>🕒 History</h3>
        <button class="btn-close" on:click={() => showHistory = false}>×</button>
      </div>

      {#if Object.values(history).every(items => items.length === 0)}
        <div class="empty-history">
          <p>No history yet</p>
          <p class="hint">Your browsing history will appear here</p>
        </div>
      {:else}
        <div class="history-list">
          {#each Object.entries(history) as [groupKey, items]}
            {#if items.length > 0}
              <div class="history-group">
                <div class="history-group-title">{groupKey}</div>
                {#each items as item}
                  <div
                    class="history-item"
                    on:click={() => navigateToHistoryItem(item)}
                    role="button"
                    tabindex="0"
                  >
                    <span class="history-type-icon">
                      {item.type === 'server' ? '🖥' : '📄'}
                    </span>
                    <div class="history-info">
                      <span class="history-name">
                        {item.type === 'server' ? item.serverName : item.topicName}
                      </span>
                      {#if item.type === 'topic'}
                        <span class="history-server-name">{item.serverName}</span>
                      {/if}
                      <span class="history-time">{formatHistoryTime(item.accessedAt)}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          {/each}
        </div>

        <div class="history-footer">
          <button class="btn-clear-history" on:click={() => { clearHistory(); loadHistory(); }}>
            Clear History
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <div class="server-list">
    {#each Array.from(servers.values()) as server (server.id)}
      <div class="server-item" class:selected={selectedServerId === server.id}>
        <div class="server-header">
          <button
            class="expand-btn"
            on:click={() => toggleServerExpansion(server.id)}
            aria-label="Toggle expansion"
          >
            {#if expandedServers.has(server.id)}
              ▼
            {:else}
              ▶
            {/if}
          </button>

          <span class="server-name">{server.name}</span>

          <span class="server-host">{server.host}:{server.port}</span>

          <button
            class="btn-fav"
            class:favorited={isServerFavorited(server.id)}
            on:click|stopPropagation={() => toggleFavoriteServer(server)}
            title={isServerFavorited(server.id) ? 'Remove from favorites' : 'Add to favorites'}
          >
            {isServerFavorited(server.id) ? '★' : '☆'}
          </button>

          <button
            class="btn-delete"
            on:click|stopPropagation={() => handleDeleteServer(server.id, server.name)}
            title="Delete server"
          >
            🗑
          </button>

          {#if serverStates.get(server.id) === 'connected'}
            <span class="status-badge connected">Connected</span>
            <button
              class="btn-disconnect"
              on:click={() => handleDisconnect(server.id)}
            >
              Disconnect
            </button>
          {:else if serverStates.get(server.id) === 'connecting'}
            <span class="status-badge connecting">Connecting...</span>
          {:else if serverStates.get(server.id) === 'error'}
            <span class="status-badge error">Error</span>
          {:else}
            <button
              class="btn-connect"
              on:click={() => handleConnect(server.id)}
            >
              Connect
            </button>
          {/if}
        </div>

        {#if expandedServers.has(server.id)}
          <div class="topics-container">
            {#if serverStates.get(server.id) === 'connected'}
              {#if serverTopics.get(server.id)}
                <div class="topics-list">
                  {#each serverTopics.get(server.id) as topic}
                    <div class="topic-item-wrapper">
                      <button
                        class="topic-item"
                        class:selected={selectedTopic === topic}
                        on:click={() => selectTopic(topic)}
                      >
                        <span class="topic-icon">📄</span>
                        <span class="topic-name">{topic}</span>
                      </button>

                      <button
                        class="btn-fav-topic"
                        class:favorited={isTopicFavorited(server.id, topic)}
                        on:click|stopPropagation={() => toggleFavoriteTopic(server.id, server.name, topic)}
                        title={isTopicFavorited(server.id, topic) ? 'Remove from favorites' : 'Add to favorites'}
                      >
                        {isTopicFavorited(server.id, topic) ? '★' : '☆'}
                      </button>
                    </div>
                  {/each}
                </div>
              {:else}
                <div class="loading-topics">Loading topics...</div>
              {/if}
            {:else}
              <div class="not-connected">Connect to view topics</div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}

    {#if servers.size === 0}
      <div class="empty-state">
        <p>No servers configured</p>
        <p class="hint">Add a server connection to get started</p>
      </div>
    {/if}
  </div>

  {#if showAddServerDialog}
    <div class="dialog-overlay" on:click={closeAddServerDialog}>
      <div class="dialog" on:click|stopPropagation>
        <div class="dialog-header">
          <h3>Add Server Connection</h3>
          <button class="btn-close" on:click={closeAddServerDialog}>×</button>
        </div>

        <div class="dialog-body">
          <div class="form-group">
            <label for="serverName">Server Name *</label>
            <input
              type="text"
              id="serverName"
              bind:value={newServerName}
              placeholder="My Kafka Cluster"
              class="form-control"
            />
          </div>

          <div class="form-group">
            <label for="bootstrapServers">Bootstrap Servers *</label>
            <input
              type="text"
              id="bootstrapServers"
              bind:value={newBootstrapServers}
              placeholder="localhost:9092"
              class="form-control"
            />
            <small>Comma-separated list of Kafka brokers</small>
          </div>

          <div class="form-group">
            <label for="securityType">Security Protocol</label>
            <select id="securityType" bind:value={newSecurityType} class="form-control">
              <option value="PLAINTEXT">PLAINTEXT</option>
              <option value="SSL">SSL</option>
              <option value="SASL_PLAINTEXT">SASL_PLAINTEXT</option>
              <option value="SASL_SSL">SASL_SSL</option>
            </select>
          </div>

          <div class="form-group">
            <label for="zookeeperHosts">ZooKeeper Hosts (optional)</label>
            <input
              type="text"
              id="zookeeperHosts"
              bind:value={newZookeeperHosts}
              placeholder="localhost:2181"
              class="form-control"
            />
            <small>For ZooKeeper-based consumers</small>
          </div>

          <div class="form-group">
            <label for="zookeeperChroot">ZooKeeper Chroot (optional)</label>
            <input
              type="text"
              id="zookeeperChroot"
              bind:value={newZookeeperChroot}
              placeholder="/kafka"
              class="form-control"
            />
            <small>ZooKeeper chroot path (e.g., /kafka)</small>
          </div>
        </div>

        <div class="dialog-footer">
          <button class="btn-secondary" on:click={closeAddServerDialog}>Cancel</button>
          <button class="btn-primary" on:click={handleAddServer}>Add Server</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .server-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #ffffff;
    border-right: 1px solid #e0e0e0;
  }

  .toolbar {
    padding: 12px;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    gap: 8px;
    background: #fafafa;
  }

  .btn-primary,
  .btn-secondary,
  .btn-close {
    padding: 8px 16px;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .btn-primary:hover {
    background: #1d4ed8;
  }

  .btn-secondary {
    background: #f3f4f6;
    color: #374151;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-secondary:hover,
  .btn-secondary.active {
    background: #e5e7eb;
  }

  .btn-secondary.active {
    background: #dbeafe;
    color: #1e40af;
  }

  .fav-icon {
    font-size: 14px;
  }

  .fav-count {
    background: #2563eb;
    color: white;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 10px;
  }

  .favorites-section {
    border-bottom: 1px solid #e0e0e0;
    background: #fefce8;
  }

  .favorites-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    border-bottom: 1px solid #fde68a;
  }

  .favorites-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #92400e;
  }

  .empty-favorites {
    padding: 24px 12px;
    text-align: center;
    color: #9ca3af;
    font-size: 13px;
  }

  .favorites-list {
    padding: 8px;
    max-height: 300px;
    overflow-y: auto;
  }

  .favorite-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: white;
    border-radius: 6px;
    margin-bottom: 6px;
    cursor: pointer;
    border: 1px solid #fde68a;
  }

  .favorite-item:hover {
    background: #fffbeb;
  }

  .fav-type-icon {
    font-size: 14px;
  }

  .fav-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .fav-name {
    font-size: 13px;
    font-weight: 500;
    color: #111827;
  }

  .fav-server-name {
    font-size: 11px;
    color: #6b7280;
  }

  .btn-remove-fav {
    padding: 4px 8px;
    background: none;
    border: none;
    font-size: 18px;
    cursor: pointer;
    color: #9ca3af;
  }

  .btn-remove-fav:hover {
    color: #dc2626;
  }

  .server-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .server-item {
    border-radius: 6px;
    margin-bottom: 4px;
    overflow: hidden;
  }

  .server-item.selected {
    background: #eff6ff;
  }

  .server-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
  }

  .server-header:hover {
    background: #f9fafb;
  }

  .expand-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    font-size: 10px;
    color: #6b7280;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .server-name {
    font-weight: 600;
    color: #111827;
    font-size: 13px;
  }

  .server-host {
    color: #6b7280;
    font-size: 12px;
    margin-right: 4px;
  }

  .btn-fav {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    font-size: 14px;
    color: #d1d5db;
  }

  .btn-fav.favorited {
    color: #f59e0b;
  }

  .btn-fav-topic {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    font-size: 12px;
    color: #d1d5db;
    margin-left: auto;
  }

  .btn-fav-topic.favorited {
    color: #f59e0b;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-badge.connected {
    background: #d1fae5;
    color: #065f46;
  }

  .status-badge.connecting {
    background: #fef3c7;
    color: #92400e;
  }

  .status-badge.error {
    background: #fee2e2;
    color: #991b1b;
  }

  .btn-connect,
  .btn-disconnect {
    padding: 4px 12px;
    border: none;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-connect {
    background: #2563eb;
    color: white;
  }

  .btn-connect:hover {
    background: #1d4ed8;
  }

  .btn-disconnect {
    background: #f3f4f6;
    color: #374151;
  }

  .btn-disconnect:hover {
    background: #e5e7eb;
  }

  .btn-delete {
    padding: 4px 8px;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    background: transparent;
    color: #9ca3af;
  }

  .btn-delete:hover {
    background: #fee2e2;
    color: #dc2626;
  }

  .topics-container {
    margin-left: 28px;
    border-left: 1px solid #e5e7eb;
    padding-left: 8px;
  }

  .topics-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .topic-item-wrapper {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .topic-item {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    background: none;
    border: none;
    width: 100%;
    text-align: left;
  }

  .topic-item:hover {
    background: #f9fafb;
  }

  .topic-item.selected {
    background: #dbeafe;
    color: #1e40af;
  }

  .topic-icon {
    font-size: 12px;
  }

  .topic-name {
    color: #374151;
  }

  .loading-topics,
  .not-connected {
    padding: 12px;
    color: #6b7280;
    font-size: 12px;
    font-style: italic;
  }

  .empty-state {
    padding: 32px 16px;
    text-align: center;
    color: #6b7280;
  }

  .hint {
    font-size: 12px;
    color: #9ca3af;
  }

  .history-icon {
    font-size: 14px;
  }

  .history-section {
    border-bottom: 1px solid #e0e0e0;
    background: #f0f9ff;
  }

  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    border-bottom: 1px solid #bae6fd;
  }

  .history-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #1d4ed8;
  }

  .empty-history {
    padding: 24px 12px;
    text-align: center;
    color: #9ca3af;
    font-size: 13px;
  }

  .history-list {
    padding: 8px;
    max-height: 400px;
    overflow-y: auto;
  }

  .history-group {
    margin-bottom: 16px;
  }

  .history-group-title {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    padding: 8px 12px 4px;
    border-bottom: 1px solid #e5e7eb;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: white;
    border-radius: 6px;
    margin-bottom: 6px;
    cursor: pointer;
    border: 1px solid #bae6fd;
  }

  .history-item:hover {
    background: #eff6ff;
  }

  .history-type-icon {
    font-size: 14px;
  }

  .history-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .history-name {
    font-size: 13px;
    font-weight: 500;
    color: #111827;
  }

  .history-server-name {
    font-size: 11px;
    color: #6b7280;
  }

  .history-time {
    font-size: 11px;
    color: #9ca3af;
  }

  .history-footer {
    padding: 12px;
    border-top: 1px solid #bae6fd;
    text-align: center;
  }

  .btn-clear-history {
    padding: 6px 16px;
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-clear-history:hover {
    background: #dc2626;
  }

  /* Dialog styles */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: white;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #e5e7eb;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #111827;
  }

  .dialog-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 500;
    color: #374151;
  }

  .form-control {
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    transition: border-color 0.2s;
  }

  .form-control:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
  }

  .form-group small {
    font-size: 11px;
    color: #6b7280;
  }

  .dialog-footer {
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 24px;
    color: #6b7280;
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .btn-close:hover {
    background: #f3f4f6;
  }
</style>
