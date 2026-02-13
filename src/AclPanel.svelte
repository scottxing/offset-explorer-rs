<script lang="ts">
  import {
    listAcls,
    createAcl,
    deleteAcl,
    type AclBinding,
    type CreateAclRequest
  } from './lib/api';

  export let serverId: number | null = null;

  // Component state
  let acls: AclBinding[] = [];
  let loading = false;
  let error: string | null = null;

  // Filter state
  let filterResourceType = '';
  let filterResourceName = '';
  let filterPrincipal = '';
  let filterOperation = '';
  let filterPermissionType = '';

  // Create dialog state
  let showCreateDialog = false;
  let newPrincipal = 'User:*';
  let newResourceType = 'Topic';
  let newResourceName = '';
  let newOperation = 'Read';
  let newPermissionType = 'Allow';
  let newHost = '*';
  let newWildcard = false;

  // Resource type options
  const resourceTypes = [
    { value: 'Topic', label: 'Topic' },
    { value: 'Group', label: 'Consumer Group' },
    { value: 'Cluster', label: 'Cluster' },
    { value: 'TransactionalId', label: 'Transactional ID' }
  ];

  // Operation options
  const operations = [
    { value: 'Read', label: 'Read' },
    { value: 'Write', label: 'Write' },
    { value: 'Create', label: 'Create' },
    { value: 'Delete', label: 'Delete' },
    { value: 'Alter', label: 'Alter' },
    { value: 'Describe', label: 'Describe' },
    { value: 'ClusterAction', label: 'Cluster Action' },
    { value: 'All', label: 'All' }
  ];

  // Permission type options
  const permissionTypes = [
    { value: 'Allow', label: 'Allow' },
    { value: 'Deny', label: 'Deny' }
  ];

  // Load ACLs
  async function loadAcls() {
    if (!serverId) return;

    loading = true;
    error = null;
    try {
      const filter = {
        resourceType: filterResourceType || undefined,
        resourceName: filterResourceName || undefined,
        principal: filterPrincipal || undefined,
        operation: filterOperation || undefined,
        permissionType: filterPermissionType || undefined
      };

      acls = await listAcls(serverId, Object.keys(filter).length > 0 ? filter : undefined);
    } catch (err) {
      error = err as string;
      console.error('Failed to load ACLs:', err);
    } finally {
      loading = false;
    }
  }

  // Create new ACL
  async function handleCreateAcl() {
    if (!serverId || !newResourceName.trim()) return;

    const request: CreateAclRequest = {
      principal: newPrincipal,
      resourceType: newResourceType,
      resourceName: newResourceName.trim(),
      operation: newOperation,
      permissionType: newPermissionType,
      host: newHost || undefined,
      wildcard: newWildcard
    };

    try {
      await createAcl(serverId, request);
      await loadAcls();
      closeCreateDialog();
    } catch (err) {
      error = err as string;
      console.error('Failed to create ACL:', err);
    }
  }

  // Delete ACL
  async function handleDeleteAcl(acl: AclBinding) {
    if (!serverId) return;
    if (!confirm(`Are you sure you want to delete this ACL?\n\nPrincipal: ${acl.principal}\nResource: ${acl.resourceType}:${acl.resourceName}\nOperation: ${acl.operation}`)) {
      return;
    }

    try {
      await deleteAcl(
        serverId,
        acl.principal,
        acl.resourceType,
        acl.resourceName,
        acl.operation,
        acl.permissionType
      );
      await loadAcls();
    } catch (err) {
      error = err as string;
      console.error('Failed to delete ACL:', err);
    }
  }

  // Open create dialog
  function openCreateDialog() {
    newPrincipal = 'User:*';
    newResourceType = 'Topic';
    newResourceName = '';
    newOperation = 'Read';
    newPermissionType = 'Allow';
    newHost = '*';
    newWildcard = false;
    showCreateDialog = true;
  }

  // Close create dialog
  function closeCreateDialog() {
    showCreateDialog = false;
  }

  // Clear filters
  function clearFilters() {
    filterResourceType = '';
    filterResourceName = '';
    filterPrincipal = '';
    filterOperation = '';
    filterPermissionType = '';
    loadAcls();
  }

  // Watch for serverId changes
  $: if (serverId) {
    loadAcls();
  }

  // Format operation for display
  function formatOperation(op: string): string {
    const ops: Record<string, string> = {
      'Read': 'Read',
      'Write': 'Write',
      'Create': 'Create',
      'Delete': 'Delete',
      'Alter': 'Alter',
      'Describe': 'Describe',
      'ClusterAction': 'Cluster Action',
      'DescribeConfigs': 'Describe Configs',
      'AlterConfigs': 'Alter Configs',
      'All': 'All'
    };
    return ops[op] || op;
  }

  // Get icon for resource type
  function getResourceIcon(type: string): string {
    const icons: Record<string, string> = {
      'Topic': '📄',
      'Group': '👥',
      'Cluster': '🏢',
      'TransactionalId': '🔄',
      'DelegationToken': '🔑'
    };
    return icons[type] || '📋';
  }
</script>

<div class="acl-panel">
  <div class="panel-header">
    <h2>ACL Management</h2>
    <button class="btn-primary" on:click={loadAcls} disabled={loading || !serverId}>
      Refresh
    </button>
    <button class="btn-primary" on:click={openCreateDialog} disabled={!serverId}>
      + New ACL
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
      <div class="loading-state">Loading ACLs...</div>
    {:else if !serverId}
      <div class="empty-state">
        <p>No server selected</p>
        <p class="hint">Connect to a server to manage ACLs</p>
      </div>
    {:else if acls.length === 0}
      <div class="empty-state">
        <p>No ACLs found</p>
        <p class="hint">Create an ACL to get started</p>
      </div>
    {:else}
      <div class="filters-section">
        <div class="filter-row">
          <input
            type="text"
            placeholder="Filter by resource name"
            bind:value={filterResourceName}
            on:change={loadAcls}
            class="filter-input"
          />
          <select bind:value={filterResourceType} on:change={loadAcls} class="filter-select">
            <option value="">All Resource Types</option>
            <option value="Topic">Topic</option>
            <option value="Group">Group</option>
            <option value="Cluster">Cluster</option>
          </select>
          <select bind:value={filterOperation} on:change={loadAcls} class="filter-select">
            <option value="">All Operations</option>
            <option value="Read">Read</option>
            <option value="Write">Write</option>
            <option value="Create">Create</option>
            <option value="Delete">Delete</option>
            <option value="Alter">Alter</option>
            <option value="All">All</option>
          </select>
          <button class="btn-secondary" on:click={clearFilters}>Clear</button>
        </div>
      </div>

      <div class="acls-table">
        <div class="table-header">
          <span class="col-principal">Principal</span>
          <span class="col-resource">Resource</span>
          <span class="col-operation">Operation</span>
          <span class="col-permission">Permission</span>
          <span class="col-host">Host</span>
          <span class="col-actions">Actions</span>
        </div>
        {#each acls as acl}
          <div class="acl-row" class:wildcard={acl.wildcard}>
            <span class="col-principal">{acl.principal}</span>
            <span class="col-resource">
              <span class="resource-icon">{getResourceIcon(acl.resourceType)}</span>
              <span class="resource-name">{acl.resourceName}</span>
              {#if acl.wildcard}
                <span class="wildcard-badge">Wildcard</span>
              {/if}
            </span>
            <span class="col-operation">{formatOperation(acl.operation)}</span>
            <span class="col-permission" class:allow={acl.permissionType === 'Allow'} class:deny={acl.permissionType === 'Deny'}>
              {acl.permissionType}
            </span>
            <span class="col-host">{acl.host}</span>
            <span class="col-actions">
              <button
                class="btn-delete"
                on:click={() => handleDeleteAcl(acl)}
                title="Delete ACL"
              >
                🗑
              </button>
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

{#if showCreateDialog}
  <div class="dialog-overlay" on:click={closeCreateDialog}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>Create New ACL</h3>
        <button class="btn-close" on:click={closeCreateDialog}>×</button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label for="principal">Principal</label>
          <input
            type="text"
            id="principal"
            bind:value={newPrincipal}
            placeholder="User:* or User:alice"
            required
          />
          <small>Format: User:* for all users or User:username for specific user</small>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="resourceType">Resource Type</label>
            <select id="resourceType" bind:value={newResourceType} required>
              {#each resourceTypes as type}
                <option value={type.value}>{type.label}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="resourceName">Resource Name</label>
            <input
              type="text"
              id="resourceName"
              bind:value={newResourceName}
              placeholder="topic-name or * for wildcard"
              required
            />
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="operation">Operation</label>
            <select id="operation" bind:value={newOperation} required>
              {#each operations as op}
                <option value={op.value}>{op.label}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="permissionType">Permission Type</label>
            <select id="permissionType" bind:value={newPermissionType} required>
              {#each permissionTypes as perm}
                <option value={perm.value}>{perm.label}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="form-group">
          <label for="host">Host</label>
          <input
            type="text"
            id="host"
            bind:value={newHost}
            placeholder="* for all hosts"
          />
          <small>* for all hosts or specific hostname</small>
        </div>

        <div class="form-group">
          <label>
            <input type="checkbox" bind:checked={newWildcard} />
            Wildcard pattern
          </label>
          <small>Use wildcard matching for resource name</small>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" on:click={closeCreateDialog}>Cancel</button>
        <button
          class="btn-primary"
          on:click={handleCreateAcl}
          disabled={!newResourceName.trim()}
        >
          Create ACL
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .acl-panel {
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

  .filters-section {
    margin-bottom: 20px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 6px;
    border: 1px solid #e5e7eb;
  }

  .filter-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .filter-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
  }

  .filter-select {
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
    background: white;
  }

  .acls-table {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    overflow: hidden;
  }

  .table-header {
    display: grid;
    grid-template-columns: 150px 200px 120px 100px 120px 80px;
    gap: 12px;
    padding: 12px 16px;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .acl-row {
    display: grid;
    grid-template-columns: 150px 200px 120px 100px 120px 80px;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid #f3f4f6;
    font-size: 13px;
    color: #374151;
    align-items: center;
  }

  .acl-row:last-child {
    border-bottom: none;
  }

  .acl-row.wildcard {
    background: #fffbeb;
  }

  .col-principal {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-resource {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .resource-icon {
    font-size: 14px;
  }

  .resource-name {
    font-weight: 500;
  }

  .wildcard-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: #f59e0b;
    color: white;
    border-radius: 10px;
    font-weight: 600;
  }

  .col-permission.allow {
    color: #059669;
    font-weight: 600;
  }

  .col-permission.deny {
    color: #dc2626;
    font-weight: 600;
  }

  .col-host {
    font-size: 12px;
    color: #6b7280;
  }

  .col-actions {
    display: flex;
    justify-content: center;
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
    max-width: 600px;
    max-height: 90vh;
    overflow-y: auto;
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

  .form-group input,
  .form-group select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 13px;
  }

  .form-group input:focus,
  .form-group select:focus {
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

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
  }
</style>
