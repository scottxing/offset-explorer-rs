<script lang="ts">
  import {
    listConsumerGroups,
    getConsumerGroupDetails,
    resetConsumerOffset,
    type ConsumerGroupResponse
  } from './lib/api';

  export let serverId: number | null = null;

  // Consumer groups state
  let consumerGroups: ConsumerGroupResponse[] = [];
  let loading = false;
  let error: string | null = null;

  // Selected consumer group details
  let selectedGroupId: string | null = null;
  let groupDetails: any = null;
  let loadingDetails = false;

  // Offset reset dialog state
  let showResetDialog = false;
  let resetTopic = '';
  let resetPartition = 0;
  let resetOffset = 0;
  let resetting = false;

  // Load consumer groups list
  async function loadConsumerGroups() {
    if (!serverId) return;

    loading = true;
    error = null;
    try {
      consumerGroups = await listConsumerGroups(serverId);
    } catch (err) {
      error = err as string;
      console.error('Failed to load consumer groups:', err);
    } finally {
      loading = false;
    }
  }

  // Load consumer group details
  async function loadGroupDetails(groupId: string) {
    if (!serverId) return;

    loadingDetails = true;
    try {
      groupDetails = await getConsumerGroupDetails(serverId, groupId);
    } catch (err) {
      console.error('Failed to load group details:', err);
    } finally {
      loadingDetails = false;
    }
  }

  // Select a consumer group
  function selectGroup(groupId: string) {
    selectedGroupId = groupId;
    loadGroupDetails(groupId);
  }

  // Open offset reset dialog
  function openResetDialog(topic: string, partition: number, currentOffset: number) {
    resetTopic = topic;
    resetPartition = partition;
    resetOffset = currentOffset;
    showResetDialog = true;
  }

  // Reset consumer offset
  async function handleResetOffset() {
    if (!serverId || !selectedGroupId) return;

    resetting = true;
    try {
      await resetConsumerOffset(
        serverId,
        selectedGroupId,
        resetTopic,
        resetPartition,
        resetOffset
      );
      await loadGroupDetails(selectedGroupId);
      showResetDialog = false;
    } catch (err) {
      error = err as string;
      console.error('Failed to reset offset:', err);
    } finally {
      resetting = false;
    }
  }

  // Get state badge color
  function getStateColor(state: string): string {
    switch (state.toLowerCase()) {
      case 'stable': return '#d1fae5';
      case 'preparingrebalance':
      case 'completingrebalance': return '#fef3c7';
      case 'dead':
      case 'empty': return '#fee2e2';
      default: return '#f3f4f6';
    }
  }

  function getStateTextColor(state: string): string {
    switch (state.toLowerCase()) {
      case 'stable': return '#065f46';
      case 'preparingrebalance':
      case 'completingrebalance': return '#92400e';
      case 'dead':
      case 'empty': return '#991b1b';
      default: return '#374151';
    }
  }

  // Calculate total lag
  function calculateTotalLag(): number {
    if (!groupDetails || !groupDetails.members) return 0;
    return groupDetails.members.reduce((total: number, member: any) => {
      return total + (member.lag || 0);
    }, 0);
  }

  // Watch for serverId changes
  $: if (serverId) {
    loadConsumerGroups();
  }
</script>

<div class="consumer-panel">
  <div class="panel-header">
    <h2>Consumer Groups</h2>
    <button class="btn-primary" on:click={loadConsumerGroups} disabled={loading || !serverId}>
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
      <div class="loading-state">Loading consumer groups...</div>
    {:else if !serverId}
      <div class="empty-state">
        <p>No server selected</p>
        <p class="hint">Connect to a server to view consumer groups</p>
      </div>
    {:else if consumerGroups.length === 0}
      <div class="empty-state">
        <p>No consumer groups found</p>
        <p class="hint">Consumer groups will appear here when consumers are active</p>
      </div>
    {:else}
      <div class="groups-grid">
        <div class="groups-list-section">
          <h3>Consumer Groups ({consumerGroups.length})</h3>
          <div class="groups-list">
            {#each consumerGroups as group}
              <div
                class="group-item"
                class:selected={selectedGroupId === group.groupId}
                on:click={() => selectGroup(group.groupId)}
                role="button"
                tabindex="0"
              >
                <span class="group-icon">👥</span>
                <div class="group-info">
                  <span class="group-name">{group.groupId}</span>
                  <span class="group-state" style="background: {getStateColor(group.state)}; color: {getStateTextColor(group.state)}">
                    {group.state}
                  </span>
                </div>
                <span class="group-members">{group.members.length} members</span>
              </div>
            {/each}
          </div>
        </div>

        {#if selectedGroupId && groupDetails}
          <div class="group-details-section">
            <div class="details-header">
              <h3>{selectedGroupId}</h3>
              {#if groupDetails.state}
                <span class="state-badge" style="background: {getStateColor(groupDetails.state)}; color: {getStateTextColor(groupDetails.state)}">
                  {groupDetails.state}
                </span>
              {/if}
            </div>

            <div class="details-stats">
              <div class="stat-item">
                <span class="stat-label">Members</span>
                <span class="stat-value">{groupDetails.members?.length || 0}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">Total Lag</span>
                <span class="stat-value">{calculateTotalLag()}</span>
              </div>
              {#if groupDetails.protocolType}
                <div class="stat-item">
                  <span class="stat-label">Protocol</span>
                  <span class="stat-value">{groupDetails.protocolType}</span>
                </div>
              {/if}
            </div>

            {#if groupDetails.members && groupDetails.members.length > 0}
              <div class="members-section">
                <h4>Members</h4>
                <div class="members-list">
                  {#each groupDetails.members as member}
                    <div class="member-card">
                      <div class="member-header">
                        <span class="member-id">{member.memberId || 'Unknown'}</span>
                        {#if member.clientId}
                          <span class="member-client">{member.clientId}</span>
                        {/if}
                        {#if member.lag !== undefined}
                          <span class="member-lag" class:high-lag={member.lag > 1000}>
                            Lag: {member.lag}
                          </span>
                        {/if}
                      </div>

                      {#if member.host}
                        <div class="member-host">Host: {member.host}</div>
                      {/if}

                      {#if member.assignments && member.assignments.length > 0}
                        <div class="assignments-list">
                          <h5>Assignments</h5>
                          {#each member.assignments as assignment}
                            <div class="assignment-item">
                              <span class="assignment-topic">{assignment.topic}</span>
                              <span class="assignment-partition">Partition: {assignment.partition}</span>
                              {#if assignment.offset !== undefined}
                                <span class="assignment-offset">Offset: {assignment.offset}</span>
                              {/if}
                              {#if assignment.lag !== undefined}
                                <span class="assignment-lag" class:high-lag={assignment.lag > 100}>
                                  Lag: {assignment.lag}
                                </span>
                              {/if}
                              <button
                                class="btn-reset"
                                on:click={() => openResetDialog(assignment.topic, assignment.partition, assignment.offset || 0)}
                                title="Reset offset"
                              >
                                Reset
                              </button>
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <div class="group-details-section empty-details">
            <p>Select a consumer group to view details</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

{#if showResetDialog}
  <div class="dialog-overlay" on:click={() => showResetDialog = false}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>Reset Consumer Offset</h3>
        <button class="btn-close" on:click={() => showResetDialog = false}>×</button>
      </div>

      <div class="dialog-body">
        <div class="reset-info">
          <p><strong>Consumer Group:</strong> {selectedGroupId}</p>
          <p><strong>Topic:</strong> {resetTopic}</p>
          <p><strong>Partition:</strong> {resetPartition}</p>
        </div>

        <div class="form-group">
          <label for="resetOffset">New Offset</label>
          <input
            type="number"
            id="resetOffset"
            bind:value={resetOffset}
            min="0"
            required
          />
          <small>Set to 0 to rewind to beginning, or a specific offset number</small>
        </div>

        <div class="warning-box">
          <span class="warning-icon">⚠</span>
          <span>
            Resetting offsets will cause consumers to reprocess messages.
            This may result in duplicate message processing.
          </span>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" on:click={() => showResetDialog = false}>Cancel</button>
        <button
          class="btn-primary btn-danger"
          on:click={handleResetOffset}
          disabled={resetting}
        >
          {resetting ? 'Resetting...' : 'Reset Offset'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .consumer-panel {
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

  .btn-danger {
    background: #dc2626;
  }

  .btn-danger:hover:not(:disabled) {
    background: #b91c1c;
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

  .groups-grid {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 20px;
    height: 100%;
  }

  .groups-list-section h3,
  .group-details-section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: #374151;
  }

  .groups-list {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    overflow: hidden;
  }

  .group-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid #f3f4f6;
    cursor: pointer;
    background: white;
  }

  .group-item:last-child {
    border-bottom: none;
  }

  .group-item:hover {
    background: #f9fafb;
  }

  .group-item.selected {
    background: #eff6ff;
  }

  .group-icon {
    font-size: 14px;
  }

  .group-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .group-name {
    font-size: 13px;
    font-weight: 500;
    color: #111827;
  }

  .group-state {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 10px;
    display: inline-block;
  }

  .group-members {
    font-size: 11px;
    color: #6b7280;
  }

  .group-details-section {
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

  .details-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .state-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 4px 8px;
    border-radius: 10px;
  }

  .details-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    background: white;
    border-radius: 4px;
    border: 1px solid #e5e7eb;
  }

  .stat-label {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .stat-value {
    font-size: 16px;
    font-weight: 600;
    color: #111827;
  }

  .members-section h4 {
    margin: 0 0 12px 0;
    font-size: 13px;
    font-weight: 600;
    color: #6b7280;
  }

  .members-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .member-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    padding: 12px;
  }

  .member-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .member-id {
    font-size: 13px;
    font-weight: 600;
    color: #111827;
  }

  .member-client {
    font-size: 12px;
    color: #6b7280;
    background: #f3f4f6;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .member-lag {
    margin-left: auto;
    font-size: 12px;
    font-weight: 500;
    color: #059669;
  }

  .member-lag.high-lag {
    color: #dc2626;
  }

  .member-host {
    font-size: 12px;
    color: #6b7280;
    margin-bottom: 8px;
  }

  .assignments-list h5 {
    margin: 0 0 8px 0;
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .assignment-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: #f9fafb;
    border-radius: 4px;
    font-size: 12px;
  }

  .assignment-topic {
    font-weight: 500;
    color: #111827;
  }

  .assignment-partition,
  .assignment-offset {
    color: #6b7280;
  }

  .assignment-lag {
    margin-left: auto;
    font-weight: 500;
    color: #059669;
  }

  .assignment-lag.high-lag {
    color: #dc2626;
  }

  .btn-reset {
    padding: 2px 8px;
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 3px;
    font-size: 11px;
    cursor: pointer;
  }

  .btn-reset:hover {
    background: #f3f4f6;
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

  .reset-info {
    margin-bottom: 16px;
    padding: 12px;
    background: #f9fafb;
    border-radius: 4px;
  }

  .reset-info p {
    margin: 4px 0;
    font-size: 13px;
    color: #374151;
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

  .warning-box {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 12px;
    background: #fef3c7;
    border: 1px solid #fde68a;
    border-radius: 4px;
    font-size: 12px;
    color: #92400e;
  }

  .warning-icon {
    font-size: 14px;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
  }
</style>
