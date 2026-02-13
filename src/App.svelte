<script lang="ts">
  import ServerTree from './ServerTree.svelte';
  import TopicPanel from './TopicPanel.svelte';
  import MessagePanel from './MessagePanel.svelte';
  import ConsumerPanel from './ConsumerPanel.svelte';
  import AclPanel from './AclPanel.svelte';

  // Application state
  let serverId: number | null = null;
  let selectedTopic: string | null = null;
  let activeTab: 'topics' | 'messages' | 'consumers' | 'acls' = 'topics';

  // Handle server selection
  function handleServerSelect(id: number | null) {
    serverId = id;
    selectedTopic = null;
  }

  // Handle topic selection
  function handleTopicSelect(topic: string | null) {
    selectedTopic = topic;
  }

  // Switch tabs
  function switchTab(tab: 'topics' | 'messages' | 'consumers' | 'acls') {
    activeTab = tab;
  }
</script>

<div class="app-container">
  <!-- Left Sidebar: Server Tree -->
  <div class="sidebar">
    <ServerTree
      bind:selectedServerId={serverId}
      bind:selectedTopic={selectedTopic}
    />
  </div>

  <!-- Main Content Area -->
  <div class="main-content">
    <!-- Tab Navigation -->
    <div class="tab-nav">
      <button
        class="tab-button"
        class:active={activeTab === 'topics'}
        on:click={() => switchTab('topics')}
      >
        <span class="tab-icon">📋</span>
        Topics
      </button>
      <button
        class="tab-button"
        class:active={activeTab === 'messages'}
        on:click={() => switchTab('messages')}
      >
        <span class="tab-icon">💬</span>
        Messages
      </button>
      <button
        class="tab-button"
        class:active={activeTab === 'consumers'}
        on:click={() => switchTab('consumers')}
      >
        <span class="tab-icon">👥</span>
        Consumers
      </button>
      <button
        class="tab-button"
        class:active={activeTab === 'acls'}
        on:click={() => switchTab('acls')}
      >
        <span class="tab-icon">🔒</span>
        ACLs
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      {#if activeTab === 'topics'}
        <TopicPanel
          {serverId}
          bind:selectedTopic
        />
      {:else if activeTab === 'messages'}
        <MessagePanel
          {serverId}
          bind:selectedTopic
        />
      {:else if activeTab === 'consumers'}
        <ConsumerPanel {serverId} />
      {:else if activeTab === 'acls'}
        <AclPanel {serverId} />
      {/if}
    </div>
  </div>
</div>

<style>
  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: #f5f5f5;
  }

  .sidebar {
    width: 320px;
    min-width: 280px;
    background: #ffffff;
    border-right: 1px solid #e0e0e0;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tab-nav {
    display: flex;
    background: #ffffff;
    border-bottom: 1px solid #e0e0e0;
    padding: 0 20px;
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 12px 20px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    color: #6b7280;
    transition: all 0.2s;
  }

  .tab-button:hover {
    color: #374151;
    background: #f9fafb;
  }

  .tab-button.active {
    color: #2563eb;
    border-bottom-color: #2563eb;
  }

  .tab-icon {
    font-size: 14px;
  }

  .tab-content {
    flex: 1;
    overflow: hidden;
  }
</style>
