<script lang="ts">
  import {
    consumeMessages,
    produceMessage,
    getTopicPartitions,
    type KafkaMessageResponse
  } from './lib/api';

  export let serverId: number | null = null;
  export let selectedTopic: string | null = null;

  // Message browsing state
  let messages: KafkaMessageResponse[] = [];
  let allMessages: KafkaMessageResponse[] = []; // Store all messages for searching
  let loadingMessages = false;
  let selectedPartition: number | null = null;
  let selectedOffset: number | null = null;
  let messageLimit = 100;

  // Search state
  let showSearch = false;
  let searchQuery = '';
  let searchField: 'all' | 'key' | 'value' = 'all';
  let searchMode: 'contains' | 'regex' = 'contains';
  let caseSensitive = false;
  let searchResults: KafkaMessageResponse[] = [];

  // Message production state
  let showProducer = false;
  let producerKey = '';
  let producerValue = '';
  let producerHeaders = '';
  let producing = false;
  let produceError: string | null = null;
  let produceSuccess = false;

  // Available partitions
  let partitions: number[] = [];

  // Errors
  let error: string | null = null;

  // Load messages
  async function loadMessages() {
    if (!serverId || !selectedTopic) return;

    loadingMessages = true;
    error = null;
    try {
      messages = await consumeMessages(
        serverId,
        selectedTopic,
        messageLimit,
        selectedPartition ?? undefined,
        selectedOffset ?? undefined
      );
      // Store all messages for searching
      allMessages = [...messages];
      searchResults = [];
    } catch (err) {
      error = err as string;
      console.error('Failed to load messages:', err);
    } finally {
      loadingMessages = false;
    }
  }

  // Search messages
  function searchMessages() {
    if (!searchQuery.trim()) {
      messages = [...allMessages];
      searchResults = [];
      return;
    }

    const query = caseSensitive ? searchQuery : searchQuery.toLowerCase();
    let results: KafkaMessageResponse[] = [];

    for (const msg of allMessages) {
      let key = caseSensitive ? (msg.key || '') : (msg.key || '').toLowerCase();
      let value = caseSensitive ? (msg.value || '') : (msg.value || '').toLowerCase();
      let matches = false;

      if (searchMode === 'regex') {
        try {
          const regex = new RegExp(searchQuery, caseSensitive ? '' : 'i');
          if (searchField === 'all' || searchField === 'key') {
            if (msg.key && regex.test(msg.key)) matches = true;
          }
          if (searchField === 'all' || searchField === 'value') {
            if (msg.value && regex.test(msg.value)) matches = true;
          }
        } catch (e) {
          console.error('Invalid regex:', e);
          return;
        }
      } else {
        // Contains mode
        if (searchField === 'all' || searchField === 'key') {
          if (key.includes(query)) matches = true;
        }
        if (searchField === 'all' || searchField === 'value') {
          if (value.includes(query)) matches = true;
        }
      }

      if (matches) {
        results.push(msg);
      }
    }

    searchResults = results;
    messages = results;
  }

  // Clear search
  function clearSearch() {
    searchQuery = '';
    messages = [...allMessages];
    searchResults = [];
  }

  // Highlight search match
  function highlightMatch(text: string | undefined): string {
    if (!text || !searchQuery) return text || '';

    const escapedQuery = searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    let regex: RegExp;

    try {
      regex = new RegExp(`(${escapedQuery})`, caseSensitive ? 'g' : 'gi');
    } catch (e) {
      return text;
    }

    return text.replace(regex, '<mark class="highlight">$1</mark>');
  }

  // Load partitions for topic
  async function loadPartitions() {
    if (!serverId || !selectedTopic) return;

    try {
      const partitionData = await getTopicPartitions(serverId, selectedTopic);
      partitions = partitionData.map(p => p.id).sort((a, b) => a - b);
      if (partitions.length > 0 && selectedPartition === null) {
        selectedPartition = partitions[0];
      }
    } catch (err) {
      console.error('Failed to load partitions:', err);
    }
  }

  // Produce message
  async function handleProduceMessage() {
    if (!serverId || !selectedTopic) return;

    producing = true;
    produceError = null;
    produceSuccess = false;

    try {
      // Parse headers
      let headers: Record<string, string> | undefined;
      if (producerHeaders.trim()) {
        headers = {};
        for (const line of producerHeaders.split('\n')) {
          const [key, ...valueParts] = line.split(':');
          if (key && valueParts.length > 0) {
            headers[key.trim()] = valueParts.join(':').trim();
          }
        }
      }

      await produceMessage(serverId, {
        topic: selectedTopic,
        key: producerKey.trim() || undefined,
        value: producerValue.trim() || undefined,
        headers
      });

      produceSuccess = true;
      producerKey = '';
      producerValue = '';
      producerHeaders = '';

      // Reload messages after producing
      await loadMessages();

      // Hide producer after short delay
      setTimeout(() => {
        produceSuccess = false;
        showProducer = false;
      }, 2000);
    } catch (err) {
      produceError = err as string;
      console.error('Failed to produce message:', err);
    } finally {
      producing = false;
    }
  }

  // Format timestamp
  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleString();
  }

  // Format message value (truncate if too long)
  function formatValue(value: string | undefined): string {
    if (!value) return '<null>';
    if (value.length > 200) return value.substring(0, 200) + '...';
    return value;
  }

  // Watch for topic changes
  $: if (selectedTopic) {
    loadPartitions();
    loadMessages();
  }

  // Reset partition when topic changes
  $: if (selectedTopic && partitions.length > 0) {
    if (selectedPartition === null || !partitions.includes(selectedPartition)) {
      selectedPartition = partitions[0];
    }
  }

  // Auto-search when search query changes
  $: if (searchQuery, searchField, searchMode, caseSensitive) {
    if (searchQuery || searchResults.length > 0) {
      searchMessages();
    }
  }
</script>

<div class="message-panel">
  <div class="panel-header">
    <h2>Messages</h2>
    <button class="btn-secondary" on:click={() => showProducer = !showProducer} disabled={!serverId || !selectedTopic}>
      {showProducer ? 'View Messages' : 'Produce Message'}
    </button>
    <button class="btn-secondary" on:click={() => showSearch = !showSearch} disabled={!serverId || !selectedTopic}>
      {showSearch ? 'Hide Search' : 'Search'}
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
    {#if !serverId || !selectedTopic}
      <div class="empty-state">
        <p>Select a topic to view messages</p>
        <p class="hint">Connect to a server and choose a topic from the Topics panel</p>
      </div>
    {:else if showProducer}
      <div class="producer-section">
        <h3>Produce Message to {selectedTopic}</h3>

        {#if produceSuccess}
          <div class="success-banner">
            <span class="success-icon">✓</span>
            <span>Message produced successfully!</span>
          </div>
        {/if}

        {#if produceError}
          <div class="error-banner-small">
            <span class="error-icon">⚠</span>
            <span>{produceError}</span>
          </div>
        {/if}

        <form on:submit|preventDefault={handleProduceMessage}>
          <div class="form-group">
            <label for="producerKey">Key (optional)</label>
            <input
              type="text"
              id="producerKey"
              bind:value={producerKey}
              placeholder="message-key"
            />
          </div>

          <div class="form-group">
            <label for="producerValue">Value</label>
            <textarea
              id="producerValue"
              bind:value={producerValue}
              placeholder="Enter message value (JSON, text, etc.)"
              rows={6}
              required
            ></textarea>
          </div>

          <div class="form-group">
            <label for="producerHeaders">Headers (optional, one per line: key:value)</label>
            <textarea
              id="producerHeaders"
              bind:value={producerHeaders}
              placeholder="Content-Type:application/json"
              rows={3}
            ></textarea>
          </div>

          <div class="form-actions">
            <button type="button" class="btn-secondary" on:click={() => showProducer = false}>
              Cancel
            </button>
            <button type="submit" class="btn-primary" disabled={producing || !producerValue.trim()}>
              {producing ? 'Producing...' : 'Produce Message'}
            </button>
          </div>
        </form>
      </div>
    {:else if showSearch}
      <div class="search-section">
        <h3>Search Messages</h3>

        <div class="search-controls">
          <div class="search-input-group">
            <input
              type="text"
              id="searchQuery"
              bind:value={searchQuery}
              placeholder="Enter search query..."
              class="search-input"
            />
            {#if searchQuery}
              <button class="btn-clear" on:click={clearSearch}>Clear</button>
            {/if}
          </div>

          <div class="search-options">
            <div class="search-option">
              <label for="searchField">Search in:</label>
              <select id="searchField" bind:value={searchField}>
                <option value="all">All Fields</option>
                <option value="key">Key Only</option>
                <option value="value">Value Only</option>
              </select>
            </div>

            <div class="search-option">
              <label for="searchMode">Mode:</label>
              <select id="searchMode" bind:value={searchMode}>
                <option value="contains">Contains</option>
                <option value="regex">Regular Expression</option>
              </select>
            </div>

            <div class="search-option">
              <label>
                <input type="checkbox" bind:checked={caseSensitive} />
                Case Sensitive
              </label>
            </div>
          </div>
        </div>

        {#if searchQuery && searchResults.length > 0}
          <div class="search-results-info">
            Found {searchResults.length} message{searchResults.length === 1 ? '' : 's'}
          </div>
        {:else if searchQuery && searchResults.length === 0}
          <div class="search-no-results">
            No messages found matching "{searchQuery}"
          </div>
        {/if}

        {#if searchQuery && searchResults.length > 0}
          <div class="messages-list">
            <div class="messages-table">
              {#each searchResults as message}
                <div class="message-row">
                  <div class="message-header">
                    <span class="message-topic">{message.topic}</span>
                    <span class="message-partition">Partition: {message.partition}</span>
                    <span class="message-offset">Offset: {message.offset}</span>
                    <span class="message-timestamp">{formatTimestamp(message.timestamp)}</span>
                  </div>

                  <div class="message-body">
                    {#if message.key}
                      <div class="message-key">
                        <span class="key-label">Key:</span>
                        <code class="key-value">{@html highlightMatch(message.key)}</code>
                      </div>
                    {/if}

                    <div class="message-value">
                      <span class="value-label">Value:</span>
                      <pre class="value-content">{@html highlightMatch(formatValue(message.value))}</pre>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="messages-list">
            <div class="empty-messages">
              <p>Enter a search query to find messages</p>
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="browser-section">
        <div class="controls">
          <div class="control-group">
            <label>Partition:</label>
            <select bind:value={selectedPartition}>
              <option value={null}>All</option>
              {#each partitions as partition}
                <option value={partition}>{partition}</option>
              {/each}
            </select>
          </div>

          <div class="control-group">
            <label>Limit:</label>
            <select bind:value={messageLimit}>
              <option value={50}>50</option>
              <option value={100}>100</option>
              <option value={500}>500</option>
              <option value={1000}>1000</option>
            </select>
          </div>

          <button class="btn-primary" on:click={loadMessages} disabled={loadingMessages}>
            {loadingMessages ? 'Loading...' : 'Refresh'}
          </button>
        </div>

        <div class="messages-list">
          {#if loadingMessages}
            <div class="loading-state">Loading messages...</div>
          {:else if messages.length === 0}
            <div class="empty-messages">
              <p>No messages found</p>
              <p class="hint">Try changing the partition or offset filter</p>
            </div>
          {:else}
            <div class="messages-table">
              {#each messages as message}
                <div class="message-row">
                  <div class="message-header">
                    <span class="message-topic">{message.topic}</span>
                    <span class="message-partition">Partition: {message.partition}</span>
                    <span class="message-offset">Offset: {message.offset}</span>
                    <span class="message-timestamp">{formatTimestamp(message.timestamp)}</span>
                  </div>

                  <div class="message-body">
                    {#if message.key}
                      <div class="message-key">
                        <span class="key-label">Key:</span>
                        <code class="key-value">{message.key}</code>
                      </div>
                    {/if}

                    <div class="message-value">
                      <span class="value-label">Value:</span>
                      <pre class="value-content">{formatValue(message.value)}</pre>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .message-panel {
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
  .btn-secondary {
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

  .btn-secondary:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner,
  .error-banner-small {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    background: #fef2f2;
    border-bottom: 1px solid #fecaca;
  }

  .error-banner-small {
    margin-bottom: 16px;
    border-radius: 4px;
    border: 1px solid #fecaca;
  }

  .error-icon {
    color: #dc2626;
    font-size: 16px;
  }

  .success-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    margin-bottom: 16px;
    background: #f0fdf4;
    border: 1px solid #bbf7d0;
    border-radius: 4px;
  }

  .success-icon {
    color: #16a34a;
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
  }

  .empty-state,
  .loading-state,
  .empty-messages {
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

  /* Producer Styles */
  .producer-section h3,
  .search-section h3 {
    margin: 0 0 20px 0;
    font-size: 16px;
    font-weight: 600;
    color: #111827;
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
  .form-group textarea {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .form-group textarea {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    resize: vertical;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }

  /* Browser Styles */
  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px;
    background: #fafafa;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    margin-bottom: 16px;
  }

  .control-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .control-group label {
    font-size: 13px;
    font-weight: 500;
    color: #374151;
  }

  .control-group select {
    padding: 6px 10px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    background: white;
  }

  .messages-list {
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    overflow: hidden;
  }

  .messages-table {
    max-height: calc(100vh - 300px);
    overflow-y: auto;
  }

  .message-row {
    border-bottom: 1px solid #f3f4f6;
  }

  .message-row:last-child {
    border-bottom: none;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-size: 12px;
  }

  .message-topic {
    font-weight: 600;
    color: #111827;
  }

  .message-partition,
  .message-offset,
  .message-timestamp {
    color: #6b7280;
  }

  .message-body {
    padding: 12px;
  }

  .message-key {
    margin-bottom: 8px;
    padding: 8px;
    background: #fef3c7;
    border-radius: 4px;
  }

  .key-label,
  .value-label {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    display: block;
    margin-bottom: 4px;
  }

  .key-value {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    color: #92400e;
  }

  .message-value {
    padding: 8px;
    background: #f3f4f6;
    border-radius: 4px;
  }

  .value-content {
    margin: 0;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
    color: #374151;
  }

  /* Search Styles */
  .search-section {
    padding: 0;
  }

  .search-controls {
    margin-bottom: 20px;
  }

  .search-input-group {
    position: relative;
    margin-bottom: 16px;
  }

  .search-input {
    width: 100%;
    padding: 12px 80px 12px 16px;
    border: 2px solid #d1d5db;
    border-radius: 8px;
    font-size: 14px;
  }

  .search-input:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
  }

  .btn-clear {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    padding: 6px 12px;
    background: #6b7280;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .btn-clear:hover {
    background: #374151;
  }

  .search-options {
    display: flex;
    gap: 16px;
    padding: 12px;
    background: #f9fafb;
    border-radius: 6px;
    flex-wrap: wrap;
  }

  .search-option {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-option label {
    font-size: 13px;
    font-weight: 500;
    color: #374151;
    white-space: nowrap;
  }

  .search-option select {
    padding: 6px 10px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    background: white;
  }

  .search-results-info {
    padding: 12px;
    background: #dbeafe;
    border: 1px solid #bfdbfe;
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 13px;
    font-weight: 500;
    color: #1e40af;
  }

  .search-no-results {
    padding: 12px;
    background: #fef3c7;
    border: 1px solid #fde68a;
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 13px;
    color: #92400e;
  }

  :global(.highlight) {
    background: #fef08a;
    border-radius: 2px;
    padding: 1px 2px;
    font-weight: 600;
  }
</style>
