<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { securityApi, securityTaskApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'incidents' | 'lostfound' | 'broadcasts' | 'findings' | 'tasks';
  let activeTab = $state<Tab>('incidents');

  let incidents: any[] = $state([]);
  let lostFound: any[] = $state([]);
  let broadcasts: any[] = $state([]);
  let findings: any[] = $state([]);
  let securityTasks: any[] = $state([]);
  let users: any[] = $state([]);
  let loading = $state(false);

  // Incident form
  let incidentOpen = $state(false);
  let incTitle = $state('');
  let incDesc = $state('');
  let incLocation = $state('');
  let incDate = $state('');
  let incSeverity = $state('');

  // Lost & Found form
  let lfOpen = $state(false);
  let lfItem = $state('');
  let lfDesc = $state('');
  let lfLocation = $state('');
  let lfDate = $state('');

  // Broadcast form
  let broadcastOpen = $state(false);
  let bcTitle = $state('');
  let bcContent = $state('');
  let bcAudience = $state('');

  // Findings form
  let findingsOpen = $state(false);
  let findTitle = $state('');
  let findDesc = $state('');
  let findDate = $state('');

  // Security task form (Guardian)
  let taskOpen = $state(false);
  let tAssignedTo = $state('');
  let tTitle = $state('');
  let tDesc = $state('');
  let tPriority = $state('medium');
  let tDue = $state('');

  // Update task form
  let updateTaskOpen = $state(false);
  let updateTaskId = $state('');
  let updateTaskStatus = $state('in_progress');
  let updateTaskNotes = $state('');

  const isGuardian = $derived($session?.role_name === 'the_guardian' || $session?.role_name === 'the_administrator');
  const isOverseer = $derived($session?.role_name === 'the_overseer');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      const tasks = [
        securityApi.getIncidents(s.token),
        securityApi.getLostFound(s.token),
        securityApi.getBroadcastRequests(s.token),
        securityApi.getFindings(s.token),
        securityTaskApi.getTasks(s.token),
        userApi.getAll(s.token),
      ];
      const [inc, lf, bc, fin, st, u] = await Promise.all(tasks);
      incidents = inc; lostFound = lf; broadcasts = bc; findings = fin; securityTasks = st; users = u;
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function createIncident() {
    const s = $session; if (!s) return;
    if (!incTitle || !incDesc) { showToast('Title and description required', 'error'); return; }
    try {
      await securityApi.createIncident(s.token, incTitle, incDesc, incLocation || undefined, incDate || undefined, incSeverity || undefined);
      showToast('Incident reported', 'success');
      incidentOpen = false; incTitle = ''; incDesc = ''; incLocation = ''; incDate = ''; incSeverity = '';
      incidents = await securityApi.getIncidents(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addLostFound() {
    const s = $session; if (!s) return;
    if (!lfItem) { showToast('Item name required', 'error'); return; }
    try {
      await securityApi.addLostFound(s.token, lfItem, lfDesc || undefined, lfLocation || undefined, lfDate || undefined);
      showToast('Item logged', 'success');
      lfOpen = false; lfItem = ''; lfDesc = ''; lfLocation = ''; lfDate = '';
      lostFound = await securityApi.getLostFound(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitBroadcast() {
    const s = $session; if (!s) return;
    if (!bcTitle || !bcContent) { showToast('Title and content required', 'error'); return; }
    try {
      await securityApi.submitBroadcastRequest(s.token, bcTitle, bcContent, bcAudience || undefined);
      showToast('Broadcast request submitted', 'success');
      broadcastOpen = false; bcTitle = ''; bcContent = ''; bcAudience = '';
      broadcasts = await securityApi.getBroadcastRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFindings() {
    const s = $session; if (!s) return;
    if (!findTitle) { showToast('Title required', 'error'); return; }
    try {
      await securityApi.submitFindings(s.token, findTitle, findDesc || undefined, findDate || undefined);
      showToast('Findings submitted', 'success');
      findingsOpen = false; findTitle = ''; findDesc = ''; findDate = '';
      findings = await securityApi.getFindings(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const incidentCols = [
    { key: 'title', label: 'Title' },
    { key: 'severity', label: 'Severity' },
    { key: 'location', label: 'Location' },
    { key: 'status', label: 'Status' },
  ];
  const lfCols = [
    { key: 'item_name', label: 'Item' },
    { key: 'found_location', label: 'Found At' },
    { key: 'found_date', label: 'Found Date' },
    { key: 'status', label: 'Status' },
  ];
  const broadcastCols = [
    { key: 'title', label: 'Title' },
    { key: 'target_audience', label: 'Audience' },
    { key: 'status', label: 'Status' },
  ];
  const findingsCols = [
    { key: 'title', label: 'Title' },
    { key: 'findings_date', label: 'Date' },
  ];
  const taskCols = [
    { key: 'title', label: 'Task' },
    { key: 'priority', label: 'Priority' },
    { key: 'status', label: 'Status' },
    { key: 'assigned_to', label: 'Assigned To' },
    { key: 'due_date', label: 'Due' },
  ];
  const severityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
  const priorityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
  const taskStatusOpts = [
    { value: 'in_progress', label: 'In Progress' },
    { value: 'result_submitted', label: 'Result Submitted' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
  let userOpts = $derived(users.map((u: any) => ({ value: u.id, label: `${u.full_name} (${u.username})` })));

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!tAssignedTo || !tTitle) { showToast('Assigned user and title required', 'error'); return; }
    try {
      await securityTaskApi.assign(s.token, tAssignedTo, tTitle, tDesc || undefined, tPriority || undefined, tDue || undefined);
      showToast('Security task assigned', 'success');
      taskOpen = false; tAssignedTo = ''; tTitle = ''; tDesc = ''; tPriority = 'medium'; tDue = '';
      securityTasks = await securityTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function updateTask() {
    const s = $session; if (!s) return;
    try {
      await securityTaskApi.update(s.token, updateTaskId, updateTaskStatus, updateTaskNotes || undefined);
      showToast('Task updated', 'success');
      updateTaskOpen = false;
      securityTasks = await securityTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }
</script>

<svelte:head><title>RUSA IMS — Security</title></svelte:head>

<PageShell title="Security Operations" subtitle="Incident reports, lost & found, broadcasts, findings, and task assignment">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='incidents'} onclick={() => activeTab='incidents'}>Incidents</button>
    <button class="tab" class:active={activeTab==='lostfound'} onclick={() => activeTab='lostfound'}>Lost & Found</button>
    {#if !isOverseer}
      <button class="tab" class:active={activeTab==='broadcasts'} onclick={() => activeTab='broadcasts'}>Broadcasts</button>
    {/if}
    <button class="tab" class:active={activeTab==='findings'} onclick={() => activeTab='findings'}>Findings</button>
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'incidents'}
    <div class="section-bar">
      <h2 class="section-title">Incident Reports</h2>
      <button class="btn-primary" onclick={() => incidentOpen = true}>+ Report Incident</button>
    </div>
    <Table columns={incidentCols} rows={incidents} />
  {:else if activeTab === 'lostfound'}
    <div class="section-bar">
      <h2 class="section-title">Lost & Found</h2>
      <button class="btn-primary" onclick={() => lfOpen = true}>+ Log Item</button>
    </div>
    <Table columns={lfCols} rows={lostFound} />
  {:else if activeTab === 'broadcasts'}
    <div class="section-bar">
      <h2 class="section-title">Broadcast Requests</h2>
      <button class="btn-primary" onclick={() => broadcastOpen = true}>+ Submit Broadcast</button>
    </div>
    <Table columns={broadcastCols} rows={broadcasts} />
  {:else if activeTab === 'findings'}
    <div class="section-bar">
      <h2 class="section-title">Security Findings</h2>
      <button class="btn-primary" onclick={() => findingsOpen = true}>+ Submit Findings</button>
    </div>
    <Table columns={findingsCols} rows={findings} />
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">Security Tasks</h2>
      {#if isGuardian}
        <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if !isGuardian}
      <p class="hint">Your assigned security tasks are listed below.</p>
    {/if}
    <Table columns={taskCols} rows={securityTasks}>
      {#snippet rowActions(row)}
        {#if row.status !== 'completed' && row.status !== 'cancelled'}
          <button class="btn-action" onclick={() => { updateTaskId = row.id; updateTaskStatus = row.status === 'pending' ? 'in_progress' : row.status; updateTaskNotes = row.result_notes || ''; updateTaskOpen = true; }}>Update</button>
        {/if}
      {/snippet}
    </Table>
  {/if}
</PageShell>

<Modal bind:open={incidentOpen} title="Report Incident">
  <div class="form">
    <Field label="Title" bind:value={incTitle} required />
    <Field label="Description" type="textarea" bind:value={incDesc} required />
    <Field label="Location" bind:value={incLocation} />
    <Field label="Date" type="date" bind:value={incDate} />
    <Field label="Severity" type="select" bind:value={incSeverity} options={severityOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => incidentOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createIncident}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={lfOpen} title="Log Lost & Found Item">
  <div class="form">
    <Field label="Item Name" bind:value={lfItem} required />
    <Field label="Description" type="textarea" bind:value={lfDesc} />
    <Field label="Found Location" bind:value={lfLocation} />
    <Field label="Found Date" type="date" bind:value={lfDate} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => lfOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addLostFound}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={broadcastOpen} title="Submit Broadcast Request">
  <div class="form">
    <Field label="Title" bind:value={bcTitle} required />
    <Field label="Content" type="textarea" bind:value={bcContent} required rows={5} />
    <Field label="Target Audience" bind:value={bcAudience} placeholder="e.g. all, security, medical" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => broadcastOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBroadcast}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={findingsOpen} title="Submit Security Findings">
  <div class="form">
    <Field label="Title" bind:value={findTitle} required />
    <Field label="Description" type="textarea" bind:value={findDesc} />
    <Field label="Findings Date" type="date" bind:value={findDate} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => findingsOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitFindings}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskOpen} title="Assign Security Task">
  <div class="form">
    {#if userOpts.length > 0}
      <Field label="Assign To" type="select" bind:value={tAssignedTo} options={userOpts} required />
    {:else}
      <Field label="Assign To (User ID)" bind:value={tAssignedTo} required />
    {/if}
    <Field label="Task Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
    <Field label="Priority" type="select" bind:value={tPriority} options={priorityOpts} />
    <Field label="Due Date" type="date" bind:value={tDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => taskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={updateTaskOpen} title="Update Security Task">
  <div class="form">
    <Field label="Status" type="select" bind:value={updateTaskStatus} options={taskStatusOpts} required />
    <Field label="Result Notes" type="textarea" bind:value={updateTaskNotes} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => updateTaskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateTask}>Update</button>
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-action { background: none; border: 1px solid #3d7fff; border-radius: 4px; color: #3d7fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.06em; padding: 0.25rem 0.75rem; }
  .btn-action:hover { background: #3d7fff22; }
  .hint { font-size: 0.78rem; color: #6a7f9a; margin: 0 0 0.75rem; }
</style>
