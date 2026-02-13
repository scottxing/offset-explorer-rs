<script lang="ts">
  import {
    getZkChildren,
    getZkNode,
    createZkNode,
    updateZkNode,
    deleteZkNode,
    type ZkNode,
    type CreateZkNodeRequest,
    type DeleteZkNodeRequest
  } from './lib/api';

  export let serverId: number | null = null;

  // Component state
  let zkNodes: ZkNode[] = [];
  let loading = false;
  let error: string | null = null;

  // Current path
  let currentPath = '/';
  let pathInput = '/';

  // Dialog states
  let showCreateDialog = false;
  let showEditDialog = false;
  let showViewDialog = false;

  // Create dialog state
  let newNodePath = '';
  let newNodeData = '';
  let newNodeIsBinary = false;
  let newNodeCreateMode = 'persistent';

  // View dialog state
  let viewNode: ZkNode | null = null;
  let viewDataDecoded: any = null;

  // Load ZK tree
  async function loadZkTree(path: string) {
    if (!serverId) return;

    loading = true;
    error = null;
    try {
      zkNodes = await getZkChildren(serverId, path);
      currentPath = path;
    } catch (err) {
      error = err as string;
      console.error('Failed to load ZK tree:', err);
    } finally {
      loading = false;
    }
  }

  // Navigate to path
  function navigateToPath(path: string) {
    pathInput = path;
    loadZkTree(path);
  }

  // Navigate up one level
  function navigateUp() {
    const parts = currentPath.split('/').filter(p => p);
    parts.pop();
    const newPath = '/' + parts.join('/');
    navigateToPath(newPath || '/');
  }

  // Open create dialog
  function openCreateDialog() {
    newNodePath = currentPath;
    newNodeData = '';
    newNodeIsBinary = false;
    newNodeCreateMode = 'persistent';
    showCreateDialog = true;
  }

  // Close create dialog
  function closeCreateDialog() {
    showCreateDialog = false;
  }

  // Create new node
  async function handleCreateNode() {
    if (!serverId || !newNodePath.trim()) return;

    const request: CreateZkNodeRequest = {
      path: newNodePath,
      data: newNodeData,
      isBinary: newNodeIsBinary,
      createMode: newNodeCreateMode,
      acl: []
    };

    try {
      await createZkNode(serverId, request);
      await loadZkTree(currentPath);
      closeCreateDialog();
    } catch (err) {
      error = err as string;
      console.error('Failed to create node:', err);
    }
  }

  // View node data
  async function handleViewNode(node: ZkNode) {
    try {
      const nodeData = await getZkNode(serverId!, node.path);

      // Decode data based on whether it's binary
      if (node.data) {
        if (node.isDirectory) {
          // Directory data is base64 encoded
          viewDataDecoded = node.data;
        } else {
          // Try to decode as UTF-8 string first
          try {
            viewDataDecoded = decodeURIComponent(escape(node.data));
          } catch {
            // If UTF-8 fails, treat as base64
            try {
              viewDataDecoded = atob(node.data);
            } catch {
              viewDataDecoded = node.data;
            }
          }
        }
      } else {
        viewDataDecoded = '<empty>';
      }

      viewNode = node;
      showViewDialog = true;
    } catch (err) {
      error = err as string;
      console.error('Failed to view node:', err);
    }
  }

  // Edit node (opens create dialog with current data)
  function handleEditNode(node: ZkNode) {
    newNodePath = node.path;
    newNodeData = node.data || '';
    newNodeIsBinary = !node.isDirectory;
    newNodeCreateMode = 'persistent';
    showCreateDialog = true;
  }

  // Delete node
  async function handleDeleteNode(node: ZkNode) {
    if (!serverId) return;
    const confirmMsg = node.isDirectory
      ? `Are you sure you want to delete directory "${node.path}" and all its children?`
      : `Are you sure you want to delete node "${node.path}"?`;

    if (!confirm(confirmMsg)) return;

    const request: DeleteZkNodeRequest = {
      path: node.path,
      recursive: node.isDirectory,
      expectedVersion: node.version
    };

    try {
      await deleteZkNode(serverId, request);
      await loadZkTree(currentPath);
    } catch (err) {
      error = err as string;
      console.error('Failed to delete node:', err);
    }
  }

  // Format timestamp
  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleString();
  }

  // Format data size
  function formatSize(size: number): string {
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    return `${(size / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Get icon for node type
  function getNodeIcon(node: ZkNode): string {
    if (node.isDirectory) {
      return '📁';
    } else {
      return '📄';
    }
  }

  // Watch for serverId changes
  $: if (serverId) {
    loadZkTree('/');
  }
</script>

<div class="zk-browser">
  <div class="panel-header">
    <h2>ZooKeeper Browser</h2>
    <button class="btn-primary" on:click={() => loadZkTree(currentPath)} disabled={loading || !serverId}>
      Refresh
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <span class="error-icon">⚠</span>
      <span>{error}</span>
      <button on:click={() => error = null} class="btn-close">×</button>
    </div>
  {/if}

  <div class="panel-content">
    {#if loading}
      <div class="loading-state">Loading ZooKeeper nodes...</div>
    {:else if !serverId}
      <div class="empty-state">
        <p>No server selected</p>
        <p class="hint">Connect to a server to browse ZooKeeper</p>
      </div>
    {:else}
      <div class="path-nav">
        <button
          class="nav-btn"
          disabled={currentPath === '/'}
          on:click={navigateUp}
        >
          ↑ Up
        </button>
        <input
          type="text"
          class="path-input"
          bind:value={pathInput}
          on:keypress={(e) => e.key === 'Enter' && navigateToPath(pathInput)}
          placeholder="Enter path (e.g., /brokers)"
        />
        <button
          class="nav-btn"
          on:click={() => navigateToPath(pathInput)}
        >
          Go
        </button>
      </div>

      <div class="zk-tree">
        {#if zkNodes.length === 0}
          <div class="empty-nodes">
            <p>No nodes found at {currentPath}</p>
            <p class="hint">Try a different path or create a new node</p>
          </div>
        {:else}
          <div class="nodes-list">
            {#each zkNodes as node}
              <div class="node-item">
                <span class="node-icon" on:click={() => node.isDirectory && navigateToPath(node.path)}>
                  {getNodeIcon(node)}
                </span>
                <span class="node-info">
                  <span class="node-name" on:click={() => node.isDirectory && navigateToPath(node.path)}>
                    {node.path.split('/').pop()}
                  </span>
                  <span class="node-meta">
                    {#if node.stat}
                      <span class="node-size" title="Size">{formatSize(node.stat.dataLength)}</span>
                      <span class="node-version" title="Version">v{node.version}</span>
                    {/if}
                  </span>
                </span>
                <span class="node-actions">
                  {#if !node.isDirectory}
                    <button
                      class="action-btn"
                      on:click={() => handleViewNode(node)}
                      title="View data"
                    >
                      👁
                    </button>
                  {/if}
                  <button
                    class="action-btn"
                    on:click={() => handleEditNode(node)}
                    title="Edit"
                  >
                    ✏️
                  </button>
                  <button
                    class="action-btn delete"
                    on:click={() => handleDeleteNode(node)}
                    title="Delete"
                  >
                    🗑
                  </button>
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

{#if showCreateDialog}
  <div class="dialog-overlay" on:click={closeCreateDialog}>
    <div class="dialog large-dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>{newNodePath === currentPath ? 'Create New Node' : 'Create Node at ' + newNodePath}</h3>
        <button class="btn-close" on:click={closeCreateDialog}>×</button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label for="nodePath">Node Path</label>
          <input
            type="text"
            id="nodePath"
            bind:value={newNodePath}
            placeholder="/path/to/node"
            required
          />
        </div>

        <div class="form-group">
          <label for="nodeData">Node Data</label>
          <textarea
            id="nodeData"
            bind:value={newNodeData}
            placeholder="Enter node data (will be encoded)"
            rows={6}
          ></textarea>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="createMode">Create Mode</label>
            <select id="createMode" bind:value={newNodeCreateMode}>
              <option value="persistent">Persistent</option>
              <option value="ephemeral">Ephemeral</option>
              <option value="sequential">Sequential</option>
              <option value="container">Container</option>
            </select>
            <small>Node persistence mode</small>
          </div>

          <div class="form-group">
            <label>
              <input type="checkbox" bind:checked={newNodeIsBinary} />
              Binary Data
            </label>
            <small>Treat data as binary instead of UTF-8 string</small>
          </div>
        </div>

        <div class="form-info">
          <p><strong>Tip:</strong> Data will be automatically encoded based on the binary option.</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" on:click={closeCreateDialog}>Cancel</button>
        <button
          class="btn-primary"
          on:click={handleCreateNode}
          disabled={!newNodePath.trim() || !newNodeData.trim()}
        >
          Create Node
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showViewDialog && viewNode}
  <div class="dialog-overlay" on:click={() => showViewDialog = false}>
    <div class="dialog large-dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>View Node Data</h3>
        <button class="btn-close" on:click={() => showViewDialog = false}>×</button>
      </div>

      <div class="dialog-body">
        <div class="view-data-info">
          <div class="info-row">
            <span class="info-label">Path:</span>
            <span class="info-value">{viewNode.path}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Type:</span>
            <span class="info-value">{viewNode.isDirectory ? 'Directory' : 'File'}</span>
          </div>
          {#if viewNode.stat}
            <div class="info-row">
              <span class="info-label">Size:</span>
              <span class="info-value">{formatSize(viewNode.stat.dataLength)}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Version:</span>
              <span class="info-value">{viewNode.version}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Created:</span>
              <span class="info-value">{formatTimestamp(viewNode.stat.ctime)}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Modified:</span>
              <span class="info-value">{formatTimestamp(viewNode.stat.mtime)}</span>
            </div>
          {/if}
        </div>

        <div class="view-data-content">
          <div class="data-label">Data:</div>
          <pre class="data-value">{viewDataDecoded}</pre>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-primary" on:click={() => showViewDialog = false}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .zk-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #ffffff;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #e0e0e0;
    background: #fafafa;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #111827;
  }

  .btn-primary,
  .btn-secondary,
  .btn-close {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    margin-left: 8px;
  }

  .btn-primary {
    background: #2563eb;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #1d4ed8;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: #f3f4f6;
    color: #374151;
  }

  .btn-secondary:hover {
    background: #e5e7eb;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    background: #fef2f2;
    border-bottom: 1px solid #fecaca;
  }

  .error-icon {
    color: #dc2626;
    font-size: 16px;
  }

  .btn-close {
    margin-left: auto;
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: #6b7280;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .loading-state,
  .empty-state,
  .empty-nodes {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: #6b7280;
    text-align: center;
  }

  .hint {
    font-size: 13px;
    color: #9ca3af;
    margin-top: 4px;
  }

  .path-nav {
    display: flex;
    gap: 8px;
    padding: 12px;
    background: #f0fdf4;
    border-radius: 6px;
    border: 1px solid #e5e7eb;
  }

  .nav-btn {
    padding: 8px 16px;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .path-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
  }

  .zk-tree {
    flex: 1;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    overflow: hidden;
  }

  .nodes-list {
    display: flex;
    flex-direction: column;
  }

  .node-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid #f3f4f6;
    transition: background 0.2s;
  }

  .node-item:hover {
    background: #f9fafb;
  }

  .node-icon {
    font-size: 18px;
    cursor: pointer;
    padding: 4px;
  }

  .node-icon:hover {
    transform: scale(1.1);
  }

  .node-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    cursor: pointer;
  }

  .node-name {
    font-size: 14px;
    font-weight: 500;
    color: #111827;
  }

  .node-meta {
    font-size: 11px;
    color: #6b7280;
  }

  .node-size,
  .node-version {
    margin-right: 8px;
  }

  .node-actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    padding: 6px 10px;
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    background: white;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: #f3f4f6;
    transform: translateY(-2px);
  }

  .action-btn.delete:hover {
    background: #fef2f2;
    border-color: #dc2626;
  }

  /* Dialog Styles */
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
    max-width: 700px;
    max-height: 85vh;
    overflow-y: auto;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
  }

  .large-dialog {
    max-width: 800px;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #e5e7eb;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #111827;
  }

  .dialog-body {
    padding: 20px;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 500;
    color: #374151;
  }

  .form-group input,
  .form-group select,
  .form-group textarea {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    font-family: 'Courier New', monospace;
  }

  .form-group textarea {
    resize: vertical;
    min-height: 100px;
  }

  .form-group input:focus,
  .form-group select:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
  }

  .form-group small {
    display: block;
    margin-top: 4px;
    font-size: 12px;
    color: #6b7280;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .form-info {
    padding: 12px;
    background: #eff6ff;
    border-radius: 4px;
    border: 1px solid #dbeafe;
    font-size: 12px;
    margin-top: 8px;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
  }

  .view-data-info {
    margin-bottom: 20px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid #f3f4f6;
  }

  .info-label {
    font-weight: 600;
    color: #374151;
    font-size: 13px;
  }

  .info-value {
    font-family: 'Courier New', monospace;
    color: #111827;
    font-size: 13px;
  }

  .view-data-content {
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    padding: 16px;
    background: #fafafa;
  }

  .data-label {
    font-weight: 600;
    margin-bottom: 8px;
    color: #374151;
  }

  .data-value {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    padding: 12px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    color: #111827;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 400px;
    overflow-y: auto;
  }
</style>
