<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { settlementApi, governanceApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'tasks' | 'nomad_tasks' | 'supply' | 'anomalies' | 'farm_reports';
  let activeTab = $state<Tab>('tasks');

  let tasks: any[] = $state([]);
  let nomadTasks: any[] = $state([]);
  let supplyReqs: any[] = $state([]);
  let anomalies: any[] = $state([]);
  let farmReports: any[] = $state([]);
  let loading = $state(false);

  // Task form
  let taskOpen = $state(false);
  let tAssignee = $state('');
  let tTitle = $state('');
  let tDesc = $state('');
  let tDue = $state('');

  let allUsers: any[] = $state([]);
  let assigneeSearch = $state('');
  let assigneeSelected: any = $state(null);

  // Update nomad task status
  let nomadTaskStatusOpen = $state(false);
  let selectedNomadTask: any = $state(null);
  let nomadTaskStatus = $state('pending');

  // Supply request
  let supplyOpen = $state(false);
  let sTitle = $state('');
  let sDesc = $state('');
  let sItems = $state('');

  // Anomaly form
  let anomalyOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aSeverity = $state('');

  // Farm report form
  let farmOpen = $state(false);
  let fTitle = $state('');
  let fContent = $state('');
  let fCropStatus = $state('');
  let fHealthNotes = $state('');

  const isCommander = $derived($session?.role_name === 'settler_commander');

  function filteredAssignees(search: string): any[] {
    const q = search.toLowerCase();
    return allUsers.filter(u =>
      u.username?.toLowerCase().includes(q) || u.full_name?.toLowerCase().includes(q)
    ).slice(0, 8);
  }

  let assigneeSuggestions = $derived(assigneeSearch.length > 0 ? filteredAssignees(assigneeSearch) : []);

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [tasks, nomadTasks, supplyReqs, anomalies, farmReports] = await Promise.all([
        settlementApi.getTasks(s.token),
        governanceApi.getNomadTasks(s.token),
        settlementApi.getSupplyRequests(s.token),
        settlementApi.getAnomalyReports(s.token),
        settlementApi.getFarmReports(s.token),
      ]);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!assigneeSelected || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await settlementApi.assignTask(s.token, assigneeSelected.id, tTitle, tDesc || undefined, tDue || undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; assigneeSelected = null; assigneeSearch = ''; tTitle = ''; tDesc = ''; tDue = '';
      tasks = await settlementApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openNomadTaskStatus(task: any) {
    selectedNomadTask = task;
    nomadTaskStatus = task.status ?? 'pending';
    nomadTaskStatusOpen = true;
  }

  async function updateNomadTaskStatus() {
    const s = $session; if (!s || !selectedNomadTask) return;
    try {
      await governanceApi.updateNomadTaskStatus(s.token, selectedNomadTask.id, nomadTaskStatus);
      showToast('Task status updated', 'success');
      nomadTaskStatusOpen = false;
      nomadTasks = await governanceApi.getNomadTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitSupply() {
    const s = $session; if (!s) return;
    if (!sTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitSupplyRequest(s.token, undefined, sTitle, sDesc || undefined, sItems || undefined);
      showToast('Supply request submitted', 'success');
      supplyOpen = false; sTitle = ''; sDesc = ''; sItems = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitAnomaly() {
    const s = $session; if (!s) return;
    if (!aTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitAnomalyReport(s.token, undefined, aTitle, aDesc || undefined, aSeverity || undefined);
      showToast('Anomaly reported', 'success');
      anomalyOpen = false; aTitle = ''; aDesc = ''; aSeverity = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFarmReport() {
    const s = $session; if (!s) return;
    if (!fTitle || !fContent) { showToast('Title and content required', 'error'); return; }
    try {
      await settlementApi.submitFarmReport(s.token, undefined, fTitle, fContent, fCropStatus || undefined, fHealthNotes || undefined);
      showToast('Farm report submitted', 'success');
      farmOpen = false; fTitle = ''; fContent = ''; fCropStatus = ''; fHealthNotes = '';
      farmReports = await settlementApi.getFarmReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'badge badge-open';
      case 'in_progress': return 'badge badge-progress';
      case 'completed': return 'badge badge-done';
      case 'cancelled': return 'badge badge-cancelled';
      default: return 'badge';
    }
  }

  const taskCols = [
    { key: 'title', label: 'Task' },
    { key: 'assigned_to_name', label: 'Assigned To' },
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due' },
  ];
  const nomadTaskStatusOpts = [
    { value: 'pending', label: 'Pending' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
  const severityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
</script>

<svelte:head><title>RUSA IMS — Settlement Operations</title></svelte:head>

<PageShell title="Settlement Operations" subtitle="Task management, supply requests, and anomaly reports">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
    {#if isCommander}
      <button class="tab" class:active={activeTab==='nomad_tasks'} onclick={() => activeTab='nomad_tasks'}>My Nomad Tasks</button>
    {/if}
    <button class="tab" class:active={activeTab==='supply'} onclick={() => activeTab='supply'}>Supply Requests</button>
    <button class="tab" class:active={activeTab==='anomalies'} onclick={() => activeTab='anomalies'}>Anomaly Reports</button>
    {#if $session?.role_name === 'farmer' || $session?.role_name === 'settler_commander'}
      <button class="tab" class:active={activeTab==='farm_reports'} onclick={() => activeTab='farm_reports'}>Farm Reports</button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">Settler Tasks</h2>
      <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
    </div>
    <Table columns={taskCols} rows={tasks} />
  {:else if activeTab === 'nomad_tasks'}
    <div class="section-bar">
      <h2 class="section-title">Tasks Assigned by The Nomad</h2>
    </div>
    {#if nomadTasks.length === 0}
      <p class="empty">No tasks have been assigned to you by The Nomad.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each nomadTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>
                  <button class="btn-small" onclick={() => openNomadTaskStatus(task)}>Update Status</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'supply'}
    <div class="section-bar">
      <h2 class="section-title">Supply Requests</h2>
      <button class="btn-primary" onclick={() => supplyOpen = true}>+ New Request</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'status',label:'Status'},{key:'items',label:'Items'}]} rows={supplyReqs} />
  {:else if activeTab === 'anomalies'}
    <div class="section-bar">
      <h2 class="section-title">Anomaly Reports</h2>
      <button class="btn-primary" onclick={() => anomalyOpen = true}>+ Report Anomaly</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'severity',label:'Severity'},{key:'status',label:'Status'}]} rows={anomalies} />
  {:else if activeTab === 'farm_reports'}
    <div class="section-bar">
      <h2 class="section-title">Farm Reports</h2>
      {#if $session?.role_name === 'farmer' || $session?.role_name === 'settler_commander'}
        <button class="btn-primary" onclick={() => farmOpen = true}>+ Submit Farm Report</button>
      {/if}
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'crop_status',label:'Crop Status'},{key:'created_at',label:'Created At'}]} rows={farmReports} />
  {/if}
</PageShell>

<Modal bind:open={taskOpen} title="Assign Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <div class="assignee-autocomplete">
        {#if assigneeSelected}
          <span class="token">{assigneeSelected.full_name || assigneeSelected.username}<button class="token-remove" onclick={() => { assigneeSelected = null; assigneeSearch = ''; }}>×</button></span>
        {:else}
          <div class="autocomplete-wrap">
            <input class="field-input" bind:value={assigneeSearch} placeholder="Search by name or username…" />
            {#if assigneeSuggestions.length > 0}
              <div class="suggestions">
                {#each assigneeSuggestions as u}
                  <button class="suggestion-item" onclick={() => { assigneeSelected = u; assigneeSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
    <Field label="Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
    <Field label="Due Date" type="date" bind:value={tDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { taskOpen = false; assigneeSelected = null; assigneeSearch = ''; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={nomadTaskStatusOpen} title="Update Nomad Task Status">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedNomadTask?.title}</strong></p>
    <Field label="Status" type="select" bind:value={nomadTaskStatus} options={nomadTaskStatusOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => nomadTaskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateNomadTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={supplyOpen} title="Submit Supply Request">
  <div class="form">
    <Field label="Title" bind:value={sTitle} required />
    <Field label="Description" type="textarea" bind:value={sDesc} />
    <Field label="Items" type="textarea" bind:value={sItems} hint="List items, one per line" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => supplyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitSupply}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={anomalyOpen} title="Report Anomaly">
  <div class="form">
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <Field label="Severity" type="select" bind:value={aSeverity} options={severityOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => anomalyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitAnomaly}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={farmOpen} title="Submit Farm Report">
  <div class="form">
    <Field label="Title" bind:value={fTitle} required />
    <Field label="Content" type="textarea" bind:value={fContent} required />
    <Field label="Crop Status" bind:value={fCropStatus} />
    <Field label="Health Check Notes" type="textarea" bind:value={fHealthNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => farmOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitFarmReport}>Submit</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .table-wrap { overflow-x: auto; }
  .data-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.85rem; }
  .data-table th { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; padding: 0.5rem 0.75rem; text-align: left; text-transform: uppercase; border-bottom: 1px solid #1e2d4a; }
  .data-table td { border-bottom: 1px solid #0d1a2e; color: #c8d8f0; padding: 0.6rem 0.75rem; }
  .data-table tr:hover td { background: rgba(61,127,255,0.04); }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .badge-cancelled { background: rgba(255,68,102,0.12); color: #ff4466; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .assignee-autocomplete { position: relative; }
  .autocomplete-wrap { position: relative; }
  .suggestions { position: absolute; top: 100%; left: 0; right: 0; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; z-index: 100; max-height: 200px; overflow-y: auto; }
  .suggestion-item { display: block; width: 100%; background: none; border: none; border-bottom: 1px solid #1e2d4a; color: #e8eeff; cursor: pointer; font-size: 0.85rem; padding: 0.5rem 0.75rem; text-align: left; }
  .suggestion-item:last-child { border-bottom: none; }
  .suggestion-item:hover { background: rgba(61,127,255,0.1); }
  .suggestion-role { color: #4a5d82; font-size: 0.75rem; }
  .token { display: inline-flex; align-items: center; gap: 0.25rem; background: rgba(61,127,255,0.15); border: 1px solid rgba(61,127,255,0.3); border-radius: 12px; color: #8fa3cc; font-size: 0.8rem; padding: 2px 8px; }
  .token-remove { background: none; border: none; color: #8fa3cc; cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 0 2px; }
  .token-remove:hover { color: #ff4466; }
</style>

