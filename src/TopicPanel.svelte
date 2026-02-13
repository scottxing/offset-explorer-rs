<script lang="ts">
  import {
    listTopics,
    createTopic,
    deleteTopic,
    getTopicMetadata,
    getTopicPartitions,
    type TopicMetadataResponse,
    type PartitionInfo
  } from './lib/api';
  import { exportMessages, type ExportOptions } from './lib/exportImport';

  export let serverId: number | null = null;
  export let selectedTopic: string | null = null;

  // Component state
  let topics: string[] = [];
  let loading = false;
  let error: string | null = null;

  // Topic metadata state
  let topicMetadata: TopicMetadataResponse | null = null;
  let partitions: PartitionInfo[] = [];

  // UI state
  let showCreateDialog = false;
  let newTopicName = '';
  let newPartitions = 1;
  let newReplicationFactor = 1;

  // Export state
  let showExportDialog = false;
  let exportFormat: 'json' | 'csv' = 'json';
  let includeHeaders = true;
  let includeKey = true;
  let includeValue = true;
  let includeTimestamp = true;
  let exporting = false;

  // Load topics list
  async function loadTopics() {
    if (!serverId) return;

    loading = true;
    error = null;
    try {
      topics = await listTopics(serverId);
    } catch (err) {
      error = err as string;
      console.error('Failed to load topics:', err);
    } finally {
      loading = false;
    }
  }

  // Load topic details
  async function loadTopicDetails(topic: string) {
    if (!serverId) return;

    try {
      topicMetadata = await getTopicMetadata(serverId, topic);
      partitions = await getTopicPartitions(serverId, topic);
    } catch (err) {
      console.error('Failed to load topic details:', err);
    }
  }

  // Create new topic
  async function handleCreateTopic() {
    if (!serverId || !newTopicName.trim()) return;

    try {
      await createTopic(serverId, {
        name: newTopicName,
        partitions: newPartitions,
        replicationFactor: newReplicationFactor
      });
      await loadTopics();
      closeCreateDialog();
    } catch (err) {
      error = err as string;
      console.error('Failed to create topic:', err);
    }
  }

  // Delete topic
  async function handleDeleteTopic(topic: string) {
    if (!serverId) return;
    if (!confirm(`Are you sure you want to delete topic "${topic}"?`)) return;

    try {
      await deleteTopic(serverId, topic);
      if (selectedTopic === topic) {
        selectedTopic = null;
        topicMetadata = null;
        partitions = [];
      }
      await loadTopics();
    } catch (err) {
      error = err as string;
      console.error('Failed to delete topic:', err);
    }
  }

  // Open create dialog
  function openCreateDialog() {
    newTopicName = '';
    newPartitions = 1;
    newReplicationFactor = 1;
    showCreateDialog = true;
  }

  // Close create dialog
  function closeCreateDialog() {
    showCreateDialog = false;
  }

  // Handle export
  async function handleExport() {
    if (!serverId || !selectedTopic) return;

    exporting = true;
    try {
      // TODO: Fetch messages from backend
      const messages: any[] = [];

      const options: ExportOptions = {
        format: exportFormat,
        includeKey,
        includeValue,
        includeHeaders,
        includeTimestamp
      };

      await exportMessages(messages, options);
      showExportDialog = false;
    } catch (err) {
      error = err as string;
      console.error('Failed to export messages:', err);
    } finally {
      exporting = false;
    }
  }

  // Watch for serverId changes
  $: if (serverId) {
    loadTopics();
  }

  // Watch for selectedTopic changes
  $: if (selectedTopic) {
    loadTopicDetails(selectedTopic);
  }
</script>

<div class="topic-panel">
  <div class="panel-header">
    <h2>Topics</h2>
    <button class="btn-primary" on:click={loadTopics} disabled={loading || !serverId}>
      Refresh
    </button>
    <button class="btn-primary" on:click={openCreateDialog} disabled={!serverId}>
      + New Topic
    </button>
    <button class="btn-secondary" on:click={() => showExportDialog = true} disabled={!serverId || !selectedTopic || topics.length === 0}>
      Export
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
      <div class="loading-state">Loading topics...</div>
    {:else if !serverId}
      <div class="empty-state">
        <p>No server selected</p>
        <p class="hint">Connect to a server to view topics</p>
      </div>
    {:else if topics.length === 0}
      <div class="empty-state">
        <p>No topics found</p>
        <p class="hint">Create a topic to get started</p>
      </div>
    {:else}
      <div class="topics-grid">
        <div class="topics-list-section">
          <h3>All Topics ({topics.length})</h3>
          <div class="topics-list">
            {#each topics as topic}
              <div
                class="topic-item"
                class:selected={selectedTopic === topic}
                on:click={() => selectedTopic = topic}
                role="button"
                tabindex="0"
              >
                <span class="topic-icon">📄</span>
                <span class="topic-name">{topic}</span>
                <button
                  class="btn-delete"
                  on:click|stopPropagation={() => handleDeleteTopic(topic)}
                  title="Delete topic"
                >
                  🗑
                </button>
              </div>
            {/each}
          </div>
        </div>

        {#if selectedTopic && topicMetadata}
          <div class="topic-details-section">
            <h3>{selectedTopic}</h3>

            <div class="metadata-section">
              <h4>Metadata</h4>
              <div class="metadata-grid">
                <div class="metadata-item">
                  <span class="metadata-label">Partitions</span>
                  <span class="metadata-value">{topicMetadata.partitionCount}</span>
                </div>
                <div class="metadata-item">
                  <span class="metadata-label">Replication Factor</span>
                  <span class="metadata-value">{topicMetadata.replicationFactor}</span>
                </div>
                <div class="metadata-item">
                  <span class="metadata-label">Internal</span>
                  <span class="metadata-value">{topicMetadata.internal ? 'Yes' : 'No'}</span>
                </div>
              </div>
            </div>

            <div class="partitions-section">
              <h4>Partitions</h4>
              <div class="partitions-table">
                <div class="table-header">
                  <span class="col-id">ID</span>
                  <span class="col-leader">Leader</span>
                  <span class="col-replicas">Replicas</span>
                  <span class="col-isr">ISR</span>
                </div>
                {#each partitions as partition}
                  <div class="partition-row">
                    <span class="col-id">{partition.id}</span>
                    <span class="col-leader">{partition.leader}</span>
                    <span class="col-replicas">{partition.replicas.join(', ')}</span>
                    <span class="col-isr">{partition.isr.join(', ')}</span>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {:else}
          <div class="topic-details-section empty-details">
            <p>Select a topic to view details</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

{#if showCreateDialog}
  <div class="dialog-overlay" on:click={closeCreateDialog}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>Create New Topic</h3>
        <button class="btn-close" on:click={closeCreateDialog}>×</button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label for="topicName">Topic Name</label>
          <input
            type="text"
            id="topicName"
            bind:value={newTopicName}
            placeholder="my-topic"
            required
          />
        </div>

        <div class="form-group">
          <label for="partitions">Partitions</label>
          <input
            type="number"
            id="partitions"
            bind:value={newPartitions}
            min="1"
            max="1000"
            required
          />
          <small>Number of partitions for the topic</small>
        </div>

        <div class="form-group">
          <label for="replication">Replication Factor</label>
          <input
            type="number"
            id="replication"
            bind:value={newReplicationFactor}
            min="1"
            max="10"
            required
          />
          <small>Number of replicas for each partition</small>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" on:click={closeCreateDialog}>Cancel</button>
        <button
          class="btn-primary"
          on:click={handleCreateTopic}
          disabled={!newTopicName.trim()}
        >
          Create Topic
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showExportDialog}
  <div class="dialog-overlay" on:click={() => showExportDialog = false}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>Export Topic Messages</h3>
        <button class="btn-close" on:click={() => showExportDialog = false}>×</button>
      </div>

      <div class="dialog-body">
        <p>Select export format:</p>
        <div class="export-options">
          <button
            class="export-option"
            class:selected={exportFormat === 'json'}
            on:click={() => exportFormat = 'json'}
          >
            JSON
          </button>
          <button
            class="export-option"
            class:selected={exportFormat === 'csv'}
            on:click={() => exportFormat = 'csv'}
          >
            CSV
          </button>
        </div>

        <div class="form-group">
          <label>
            <input type="checkbox" bind:checked={includeHeaders} />
            Include headers (Key, Partition, Offset, Timestamp)
          </label>
          <label>
            <input type="checkbox" bind:checked={includeKey} />
            Include Key
          </label>
          <label>
            <input type="checkbox" bind:checked={includeValue} />
            Include Value
          </label>
        </div>

        <div class="form-actions">
          <button class="btn-secondary" on:click={() => showExportDialog = false}>
            Cancel
          </button>
          <button
            class="btn-primary"
            on:click={handleExport}
            disabled={exporting}
          >
            {exporting ? 'Exporting...' : 'Export'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .topic-panel {
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
  }

  .loading-state,
  .empty-state {
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

  .topics-grid {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 20px;
    height: 100%;
  }

  .topics-list-section h3,
  .topic-details-section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: #374151;
  }

  .topics-list {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    overflow: hidden;
  }

  .topic-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid #f3f4f6;
    cursor: pointer;
    background: white;
  }

  .topic-item:last-child {
    border-bottom: none;
  }

  .topic-item:hover {
    background: #f9fafb;
  }

  .topic-item.selected {
    background: #eff6ff;
  }

  .topic-icon {
    font-size: 14px;
  }

  .topic-name {
    flex: 1;
    font-size: 13px;
    color: #374151;
  }

  .btn-delete {
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 14px;
    opacity: 0.5;
  }

  .btn-delete:hover {
    opacity: 1;
  }

  .topic-details-section {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    padding: 16px;
    background: #fafafa;
  }

  .empty-details {
    display: flex;
    align-items: center;
    justify-content: center;
    color: #9ca3af;
    font-style: italic;
  }

  .metadata-section,
  .partitions-section {
    margin-bottom: 20px;
  }

  .metadata-section h4,
  .partitions-section h4 {
    margin: 0 0 12px 0;
    font-size: 13px;
    font-weight: 600;
    color: #6b7280;
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .metadata-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    background: white;
    border-radius: 4px;
    border: 1px solid #e5e7eb;
  }

  .metadata-label {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .metadata-value {
    font-size: 14px;
    font-weight: 500;
    color: #111827;
  }

  .partitions-table {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
  }

  .table-header {
    display: grid;
    grid-template-columns: 80px 1fr 2fr 2fr;
    gap: 12px;
    padding: 10px 12px;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .partition-row {
    display: grid;
    grid-template-columns: 80px 1fr 2fr 2fr;
    gap: 12px;
    padding: 10px 12px;
    border-bottom: 1px solid #f3f4f6;
    font-size: 13px;
    color: #374151;
  }

  .partition-row:last-child {
    border-bottom: none;
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
    max-width: 500px;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
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

  .form-group input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
  }

  .form-group input:focus {
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

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
  }

  .export-options {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
  }

  .export-option {
    flex: 1;
    padding: 10px 16px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    background: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .export-option:hover {
    border-color: #2563eb;
    background: #eff6ff;
  }

  .export-option.selected {
    background: #2563eb;
    color: white;
    border-color: #2563eb;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }
</style>
