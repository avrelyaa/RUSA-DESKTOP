<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import UserAutocompleteSingle from '$lib/components/UserAutocompleteSingle.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { securityApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'incidents' | 'lostfound' | 'broadcasts' | 'findings' | 'tasks' | 'external_reports';
  type ExternalStatus = 'PENDING_REVIEW' | 'UNDER_INVESTIGATION' | 'CLOSED_RESOLVED' | 'DISMISSED';
  let activeTab = $state<Tab>('incidents');

  let incidents: any[] = $state([]);
  let lostFound: any[] = $state([]);
  let broadcasts: any[] = $state([]);
  let findings: any[] = $state([]);
  let securityTasks: any[] = $state([]);
  let externalReports: any[] = $state([]);
  let allUsers: any[] = $state([]);
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
  let bcFilterPersonnel = $state<string[]>([]);
  let bcFilterDepts = $state<string[]>([]);
  let bcFilterLocations = $state<string[]>([]);

  function addBcFilter(type: string) {
    if (type === 'personnel') bcFilterPersonnel = [...bcFilterPersonnel, ''];
    if (type === 'dept') bcFilterDepts = [...bcFilterDepts, ''];
    if (type === 'loc') bcFilterLocations = [...bcFilterLocations, ''];
  }
  function removeBcFilter(type: string, i: number) {
    if (type === 'personnel') bcFilterPersonnel = bcFilterPersonnel.filter((_, idx) => idx !== i);
    if (type === 'dept') bcFilterDepts = bcFilterDepts.filter((_, idx) => idx !== i);
    if (type === 'loc') bcFilterLocations = bcFilterLocations.filter((_, idx) => idx !== i);
  }

  // Findings form
  let findingsOpen = $state(false);
  let findTitle = $state('');
  let findDesc = $state('');
  let findDate = $state('');

  // External report form
  let externalOpen = $state(false);
  let exTitle = $state('');
  let exDesc = $state('');
  let exReporterName = $state('');
  let exSecurityType = $state('');

  // External report detail / action state
  let extDetailOpen = $state(false);
  let extStatusOpen = $state(false);
  let extPublicUpdateOpen = $state(false);
  let selectedExtReport: any = $state(null);
  let extNewStatus = $state('');
  let extClosedReason = $state('');
  let extPublicMsg = $state('');
  let extEvents: any[] = $state([]);
  let extStatusView: any = $state(null);
  let extStatusViewOpen = $state(false);
  let extPublicToken = $state('');

  // Task assignment form (guardian/overseer)
  let taskOpen = $state(false);
  let taskTitle = $state('');
  let taskDesc = $state('');
  let taskDue = $state('');
  let taskAssigneeSelected: any = $state(null);

  // Task status update form (security staff)
  let taskStatusOpen = $state(false);
  let selectedTask: any = $state(null);
  let updatedStatus = $state('pending');

  const isTaskManager = $derived(
    canPerform($session, 'the_guardian') || canPerform($session, 'the_overseer')
  );
  const isSecurityStaff = $derived(
    canPerform($session, 'the_guardian') ||
    canPerform($session, 'the_overseer') ||
    canPerform($session, 'earth_security_head') ||
    canPerform($session, 'earth_security_staff') ||
    canPerform($session, 'galactic_security_head') ||
    canPerform($session, 'galactic_security_staff')
  );

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    const isManager = canPerform(s, 'the_guardian') || canPerform(s, 'the_overseer');
    try {
      const promises: Promise<any>[] = [
        securityApi.getIncidents(s.token),
        securityApi.getLostFound(s.token),
        securityApi.getBroadcastRequests(s.token),
        securityApi.getFindings(s.token),
        securityApi.getSecurityTasks(s.token),
        securityApi.getExternalReports(s.token),
      ];
      if (isManager) promises.push(userApi.getAll(s.token));
      const results = await Promise.all(promises);
      [incidents, lostFound, broadcasts, findings, securityTasks, externalReports] = results as any[];
      if (isManager) allUsers = results[6] as any[];
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!taskAssigneeSelected || !taskTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await securityApi.assignTask(s.token, taskAssigneeSelected.id, taskTitle, taskDesc || undefined, taskDue || undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; taskAssigneeSelected = null; taskTitle = ''; taskDesc = ''; taskDue = '';
      securityTasks = await securityApi.getSecurityTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openTaskStatus(task: any) {
    selectedTask = task;
    updatedStatus = task.status ?? 'pending';
    taskStatusOpen = true;
  }

  async function updateTaskStatus() {
    const s = $session; if (!s || !selectedTask) return;
    try {
      await securityApi.updateSecurityTaskStatus(s.token, selectedTask.id, updatedStatus);
      showToast('Task status updated', 'success');
      taskStatusOpen = false;
      securityTasks = await securityApi.getSecurityTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createIncident() {
    const s = $session; if (!s) return;
    if (!incTitle || !incDesc) { showToast('Title and description required', 'error'); return; }
    try {
      await securityApi.createIncident(s.token, incTitle, incDesc, incLocation || undefined, incDate ? incDate + 'T00:00:00Z' : undefined, incSeverity || undefined);
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
      const filters = {
        personnel: bcFilterPersonnel.filter(v => v.trim() !== ''),
        departments: bcFilterDepts.filter(v => v.trim() !== ''),
        locations: bcFilterLocations.filter(v => v.trim() !== '')
      };
      const hasFilters = filters.personnel.length > 0 || filters.departments.length > 0 || filters.locations.length > 0;
      await securityApi.submitBroadcastRequest(
        s.token, 
        bcTitle, 
        bcContent, 
        bcAudience || undefined,
        hasFilters ? filters : undefined
      );
      showToast('Broadcast request submitted', 'success');
      broadcastOpen = false; bcTitle = ''; bcContent = ''; bcAudience = '';
      bcFilterPersonnel = []; bcFilterDepts = []; bcFilterLocations = [];
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

  async function submitExternalReport() {
    const s = $session; if (!s) return;
    if (!exTitle || !exDesc) { showToast('Title and description required', 'error'); return; }
    try {
      const result = await securityApi.submitExternalReport(s.token, exTitle, exDesc, exReporterName || undefined, exSecurityType || undefined);
      extPublicToken = result.public_token;
      extStatusViewOpen = false;
      showToast('Report submitted. Your tracking token: ' + result.public_token, 'success');
      externalOpen = false; exTitle = ''; exDesc = ''; exReporterName = ''; exSecurityType = '';
      externalReports = await securityApi.getExternalReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openExtDetail(report: any) {
    const s = $session; if (!s) return;
    selectedExtReport = report;
    extEvents = [];
    extDetailOpen = true;
    if (isSecurityStaff) {
      try { extEvents = await securityApi.getExternalReportEvents(s.token, report.id); } catch {}
    }
  }

  async function openExtStatus(report: any, newStatus: string) {
    selectedExtReport = report;
    extNewStatus = newStatus;
    extClosedReason = '';
    extPublicMsg = '';
    extStatusOpen = true;
  }

  async function applyExtStatus() {
    const s = $session; if (!s || !selectedExtReport) return;
    try {
      await securityApi.updateExternalReportStatus(s.token, selectedExtReport.id, extNewStatus, extClosedReason || undefined, extPublicMsg || undefined);
      showToast('Status updated', 'success');
      extStatusOpen = false; extDetailOpen = false;
      externalReports = await securityApi.getExternalReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function openExtPublicUpdate(report: any) {
    selectedExtReport = report;
    extPublicMsg = report.public_update_message || '';
    extPublicUpdateOpen = true;
  }

  async function saveExtPublicUpdate() {
    const s = $session; if (!s || !selectedExtReport) return;
    if (!extPublicMsg.trim()) { showToast('Message required', 'error'); return; }
    try {
      await securityApi.addExternalReportPublicUpdate(s.token, selectedExtReport.id, extPublicMsg);
      showToast('Public update saved', 'success');
      extPublicUpdateOpen = false; extDetailOpen = false;
      externalReports = await securityApi.getExternalReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function lookupReporterStatus() {
    const s = $session; if (!s) return;
    if (!extPublicToken.trim()) { showToast('Enter your tracking token', 'error'); return; }
    try {
      extStatusView = await securityApi.getExternalReportStatus(s.token, extPublicToken.trim());
      extStatusViewOpen = true;
    } catch (e: any) { showToast('Not found or invalid token', 'error'); }
  }

  function extStatusLabel(status: string | undefined) {
    switch (status) {
      case 'PENDING_REVIEW':      return 'Pending Review';
      case 'UNDER_INVESTIGATION': return 'Under Investigation';
      case 'CLOSED_RESOLVED':     return 'Closed / Resolved';
      case 'DISMISSED':           return 'Dismissed';
      default:                    return status ?? '—';
    }
  }

  function extStatusBadgeClass(status: string | undefined) {
    switch (status) {
      case 'PENDING_REVIEW':      return 'badge-open';
      case 'UNDER_INVESTIGATION': return 'badge-progress';
      case 'CLOSED_RESOLVED':     return 'badge-done';
      case 'DISMISSED':           return 'badge-dismissed';
      default:                    return 'badge-open';
    }
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
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due Date' },
  ];
  const externalReportCols = [
    { key: 'title', label: 'Title' },
    { key: 'reporter_name', label: 'Reporter' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Submitted' },
  ];
  const severityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
  const securityTypeOpts = [
    { value: 'earth', label: 'Earth Security' },
    { value: 'galactic', label: 'Galactic Security' },
  ];
</script>

<svelte:head><title>RUSA IMS — Security</title></svelte:head>

<PageShell title="Security Operations" subtitle="Incident reports, lost & found, broadcasts, security findings, tasks, and external reports">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='incidents'} onclick={() => activeTab='incidents'}>Incidents</button>
    <button class="tab" class:active={activeTab==='lostfound'} onclick={() => activeTab='lostfound'}>Lost & Found</button>
    <button class="tab" class:active={activeTab==='broadcasts'} onclick={() => activeTab='broadcasts'}>Broadcasts</button>
    <button class="tab" class:active={activeTab==='findings'} onclick={() => activeTab='findings'}>Findings</button>
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
    <button class="tab" class:active={activeTab==='external_reports'} onclick={() => activeTab='external_reports'}>External Reports</button>
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
      <h2 class="section-title">{isTaskManager ? 'Tasks I Assigned' : 'My Assigned Tasks'}</h2>
      {#if isTaskManager}
        <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if securityTasks.length === 0}
      <p class="empty">{isTaskManager ? 'No tasks assigned yet.' : 'No tasks have been assigned to you.'}</p>
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
            {#each securityTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class="badge {task.status === 'completed' ? 'badge-done' : task.status === 'in_progress' ? 'badge-progress' : 'badge-open'}">{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>
                  {#if !isTaskManager}
                    <button class="btn-small" onclick={() => openTaskStatus(task)}>Update Status</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if activeTab === 'external_reports'}
    <div class="section-bar">
      <h2 class="section-title">External Reports</h2>
      <div class="section-actions">
        {#if !isSecurityStaff}
          <button class="btn-primary" onclick={() => externalOpen = true}>+ Submit Report to Security</button>
        {/if}
      </div>
    </div>
    {#if !isSecurityStaff}
      <div class="tracker-bar">
        <span class="tracker-label">Track existing report:</span>
        <input class="field-input tracker-input" bind:value={extPublicToken} placeholder="Paste your tracking token…" />
        <button class="btn-small" onclick={lookupReporterStatus}>Track</button>
      </div>
    {/if}
    {#if isSecurityStaff}
      <p class="info-hint">Reports submitted by non-security personnel appear here. Review and action each report below.</p>
    {/if}
    {#if externalReports.length === 0}
      <p class="empty">No external reports yet.</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Reporter</th>
              <th>Status</th>
              <th>Submitted</th>
              {#if isSecurityStaff}<th>Actions</th>{/if}
            </tr>
          </thead>
          <tbody>
            {#each externalReports as report}
              <tr>
                <td>
                  <button class="link-btn" onclick={() => openExtDetail(report)}>{report.title}</button>
                </td>
                <td>{report.reporter_name || '—'}</td>
                <td>
                  <span class="badge {extStatusBadgeClass(report.status)}">{extStatusLabel(report.status)}</span>
                </td>
                <td>{report.created_at ? new Date(report.created_at).toLocaleDateString() : '—'}</td>
                {#if isSecurityStaff}
                  <td class="action-cell">
                    {#if report.status === 'PENDING_REVIEW'}
                      <button class="btn-small" onclick={() => openExtStatus(report, 'UNDER_INVESTIGATION')}>Start Investigation</button>
                      <button class="btn-small btn-warn" onclick={() => openExtStatus(report, 'DISMISSED')}>Dismiss</button>
                    {:else if report.status === 'UNDER_INVESTIGATION'}
                      <button class="btn-small btn-success" onclick={() => openExtStatus(report, 'CLOSED_RESOLVED')}>Resolve</button>
                      <button class="btn-small btn-warn" onclick={() => openExtStatus(report, 'DISMISSED')}>Dismiss</button>
                    {/if}
                    {#if report.status !== 'CLOSED_RESOLVED' && report.status !== 'DISMISSED'}
                      <button class="btn-small" onclick={() => openExtPublicUpdate(report)}>Add Update</button>
                    {/if}
                  </td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
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
    <Field label="Subject (Title)" bind:value={bcTitle} required />
    <Field label="Message Body" type="textarea" bind:value={bcContent} required rows={5} />
    <Field label="Target Audience Description" bind:value={bcAudience} placeholder="e.g. all, security, medical" />
    
    <div class="filters-section">
      <p class="section-subtitle">Target Audience Filters (Optional)</p>
      
      <div class="filter-group">
        <div class="filter-head">
          <span>Specific Personnel (User IDs)</span>
          <button class="btn-small" onclick={() => addBcFilter('personnel')}>+ Add</button>
        </div>
        {#each bcFilterPersonnel as p, i}
          <div class="filter-row">
            <input class="field-input" bind:value={bcFilterPersonnel[i]} placeholder="User ID" />
            <button class="btn-small btn-danger" onclick={() => removeBcFilter('personnel', i)}>X</button>
          </div>
        {/each}
      </div>

      <div class="filter-group">
        <div class="filter-head">
          <span>Groups / Departments</span>
          <button class="btn-small" onclick={() => addBcFilter('dept')}>+ Add</button>
        </div>
        {#each bcFilterDepts as d, i}
          <div class="filter-row">
            <input class="field-input" bind:value={bcFilterDepts[i]} placeholder="Department name" />
            <button class="btn-small btn-danger" onclick={() => removeBcFilter('dept', i)}>X</button>
          </div>
        {/each}
      </div>

      <div class="filter-group">
        <div class="filter-head">
          <span>Planets / Sectors</span>
          <button class="btn-small" onclick={() => addBcFilter('loc')}>+ Add</button>
        </div>
        {#each bcFilterLocations as l, i}
          <div class="filter-row">
            <input class="field-input" bind:value={bcFilterLocations[i]} placeholder="Planet/Sector name" />
            <button class="btn-small btn-danger" onclick={() => removeBcFilter('loc', i)}>X</button>
          </div>
        {/each}
      </div>
    </div>

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

<Modal bind:open={externalOpen} title="Submit Report to Security">
  <div class="form">
    <p class="info-text">Use this form to report a security finding or concern. Your report will be reviewed by the security team. You will receive a <strong>tracking token</strong> after submission to follow the status of your report.</p>
    <Field label="Your Name (optional)" bind:value={exReporterName} placeholder="How should we address you?" />
    <Field label="Title" bind:value={exTitle} required />
    <Field label="Description" type="textarea" bind:value={exDesc} required />
    <Field label="Security Team" type="select" bind:value={exSecurityType} options={securityTypeOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => externalOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitExternalReport}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskOpen} title="Assign Security Task">
  <div class="form">
    <div class="field">
      <label class="field-label">Assign To</label>
      <UserAutocompleteSingle users={allUsers} bind:selected={taskAssigneeSelected} />
    </div>
    <Field label="Task Title" bind:value={taskTitle} required />
    <Field label="Description" type="textarea" bind:value={taskDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={taskDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { taskOpen = false; taskAssigneeSelected = null; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={taskStatusOpen} title="Update Task Status">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedTask?.title}</strong></p>
    <Field label="Status" type="select" bind:value={updatedStatus} options={[
      { value: 'pending', label: 'Pending' },
      { value: 'in_progress', label: 'In Progress' },
      { value: 'completed', label: 'Completed' },
      { value: 'cancelled', label: 'Cancelled' },
    ]} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => taskStatusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateTaskStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={extDetailOpen} title="External Report Detail">
  {#if selectedExtReport}
    <div class="form">
      <div class="detail-row"><span class="detail-label">Title</span><span>{selectedExtReport.title}</span></div>
      <div class="detail-row"><span class="detail-label">Reporter</span><span>{selectedExtReport.reporter_name || '—'}</span></div>
      <div class="detail-row"><span class="detail-label">Status</span><span class="badge {extStatusBadgeClass(selectedExtReport.status)}">{extStatusLabel(selectedExtReport.status)}</span></div>
      <div class="detail-row"><span class="detail-label">Submitted</span><span>{selectedExtReport.created_at ? new Date(selectedExtReport.created_at).toLocaleString() : '—'}</span></div>
      {#if selectedExtReport.status_changed_at}
        <div class="detail-row"><span class="detail-label">Status Changed</span><span>{new Date(selectedExtReport.status_changed_at).toLocaleString()}</span></div>
      {/if}
      {#if selectedExtReport.public_update_message}
        <div class="detail-row"><span class="detail-label">Public Update</span><span class="public-msg">{selectedExtReport.public_update_message}</span></div>
      {/if}
      {#if isSecurityStaff && selectedExtReport.description}
        <div class="detail-section"><span class="detail-label">Description</span><p class="detail-body">{selectedExtReport.description}</p></div>
      {/if}
      {#if isSecurityStaff && selectedExtReport.closed_reason}
        <div class="detail-row"><span class="detail-label">Internal Reason</span><span>{selectedExtReport.closed_reason}</span></div>
      {/if}
      {#if isSecurityStaff && extEvents.length > 0}
        <div class="timeline">
          <p class="detail-label">Event Timeline</p>
          {#each extEvents as ev}
            <div class="timeline-item">
              <span class="tl-dot"></span>
              <div class="tl-body">
                <span class="tl-event">{ev.event_type}</span>
                {#if ev.from_status && ev.to_status}
                  <span class="tl-transition">{extStatusLabel(ev.from_status)} → {extStatusLabel(ev.to_status)}</span>
                {/if}
                <span class="tl-time">{ev.created_at ? new Date(ev.created_at).toLocaleString() : ''}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</Modal>

<Modal bind:open={extStatusOpen} title="Update Report Status">
  {#if selectedExtReport}
    <div class="form">
      <p class="info-text">Changing status: <strong>{extStatusLabel(selectedExtReport.status)}</strong> → <strong>{extStatusLabel(extNewStatus)}</strong></p>
      {#if extNewStatus === 'DISMISSED'}
        <Field label="Dismissal Reason (required)" type="textarea" bind:value={extClosedReason} required />
      {/if}
      <Field label="Public Update Message (optional)" type="textarea" bind:value={extPublicMsg} placeholder="Reporter-visible message about this status change…" />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => extStatusOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={applyExtStatus}>Confirm</button>
      </div>
    </div>
  {/if}
</Modal>

<Modal bind:open={extPublicUpdateOpen} title="Add Public Update">
  {#if selectedExtReport}
    <div class="form">
      <p class="info-text">This message will be visible to the reporter when they check their report status.</p>
      <Field label="Message" type="textarea" bind:value={extPublicMsg} required />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => extPublicUpdateOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={saveExtPublicUpdate}>Save</button>
      </div>
    </div>
  {/if}
</Modal>

<Modal bind:open={extStatusViewOpen} title="Your Report Status">
  {#if extStatusView}
    <div class="form">
      <div class="detail-row"><span class="detail-label">Title</span><span>{extStatusView.title}</span></div>
      <div class="detail-row"><span class="detail-label">Status</span><span class="badge {extStatusBadgeClass(extStatusView.status)}">{extStatusLabel(extStatusView.status)}</span></div>
      <div class="detail-row"><span class="detail-label">Submitted</span><span>{extStatusView.created_at ? new Date(extStatusView.created_at).toLocaleString() : '—'}</span></div>
      {#if extStatusView.status_changed_at}
        <div class="detail-row"><span class="detail-label">Last Update</span><span>{new Date(extStatusView.status_changed_at).toLocaleString()}</span></div>
      {/if}
      {#if extStatusView.public_update_message}
        <div class="detail-row"><span class="detail-label">Message from Security</span><span class="public-msg">{extStatusView.public_update_message}</span></div>
      {/if}
      <div class="status-guide">
        <p class="detail-label">What this means:</p>
        {#if extStatusView.status === 'PENDING_REVIEW'}
          <p class="status-desc">We have received your report and will review it shortly.</p>
        {:else if extStatusView.status === 'UNDER_INVESTIGATION'}
          <p class="status-desc">Our security team has reviewed your submission and started an investigation.</p>
        {:else if extStatusView.status === 'CLOSED_RESOLVED'}
          <p class="status-desc">This report has been resolved. Thank you for helping improve security.</p>
        {:else if extStatusView.status === 'DISMISSED'}
          <p class="status-desc">We reviewed this report and determined no actionable security issue at this time.</p>
        {/if}
      </div>
    </div>
  {/if}
</Modal>


<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .empty { color: #4a5d82; padding: 1rem 0.75rem; text-align: center; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .info-hint { font-size: 0.8rem; color: #4a5d82; font-style: italic; margin-bottom: 0.75rem; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-small { background: none; border: 1px solid #1e2d4a; border-radius: 3px; color: #8fa3cc; cursor: pointer; font-size: 0.75rem; padding: 0.25rem 0.625rem; }
  .btn-small:hover { border-color: #3d7fff; color: #3d7fff; }
  .table-wrap { overflow-x: auto; }
  .section-actions { display: flex; gap: 0.5rem; align-items: center; }
  .action-cell { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  .link-btn { background: none; border: none; color: #00d4ff; cursor: pointer; font-size: 0.85rem; padding: 0; text-align: left; }
  .link-btn:hover { text-decoration: underline; }
  .badge-dismissed { background: rgba(255,100,100,0.15); color: #ff6464; border-radius: 3px; padding: 0.2rem 0.5rem; font-size: 0.7rem; font-family: 'Space Mono', monospace; }
  .btn-warn { border-color: #ff964a; color: #ff964a; }
  .btn-warn:hover { background: rgba(255,150,74,0.1); }
  .btn-success { border-color: #00d4a0; color: #00d4a0; }
  .btn-success:hover { background: rgba(0,212,160,0.1); }
  .tracker-bar { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1rem; }
  .tracker-label { color: #8fa3cc; font-size: 0.75rem; white-space: nowrap; }
  .tracker-input { flex: 1; min-width: 0; }
  .field-input { background: #0a0e1a; border: 1px solid #1e2d4a; border-radius: 4px; color: #e0e8ff; font-size: 0.85rem; padding: 0.5rem 0.75rem; width: 100%; box-sizing: border-box; }
  .detail-row { display: flex; gap: 1rem; align-items: flex-start; padding: 0.3rem 0; border-bottom: 1px solid #0d1626; }
  .detail-label { color: #4a5d82; font-size: 0.72rem; font-family: 'Space Mono', monospace; text-transform: uppercase; letter-spacing: 0.06em; min-width: 7rem; }
  .detail-section { padding: 0.5rem 0; }
  .detail-body { color: #8fa3cc; font-size: 0.85rem; margin: 0.25rem 0 0; }
  .public-msg { color: #8fa3cc; font-style: italic; font-size: 0.85rem; }
  .timeline { margin-top: 1rem; }
  .timeline-item { display: flex; gap: 0.75rem; align-items: flex-start; padding: 0.5rem 0; border-left: 2px solid #1e2d4a; margin-left: 0.5rem; padding-left: 1rem; }
  .tl-dot { width: 8px; height: 8px; background: #3d7fff; border-radius: 50%; flex-shrink: 0; margin-top: 0.25rem; }
  .tl-body { display: flex; flex-direction: column; gap: 0.15rem; }
  .tl-event { color: #e0e8ff; font-size: 0.8rem; font-family: 'Space Mono', monospace; }
  .tl-transition { color: #00d4ff; font-size: 0.75rem; }
  .tl-time { color: #4a5d82; font-size: 0.72rem; }
  .status-guide { background: rgba(61,127,255,0.06); border-left: 3px solid #3d7fff; border-radius: 4px; margin-top: 0.75rem; padding: 0.75rem 1rem; }
  .status-desc { color: #8fa3cc; font-size: 0.85rem; margin: 0.25rem 0 0; }
  .data-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.85rem; }
  .data-table th { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.08em; padding: 0.5rem 0.75rem; text-align: left; text-transform: uppercase; border-bottom: 1px solid #1e2d4a; }
  .data-table td { border-bottom: 1px solid #0d1a2e; color: #c8d8f0; padding: 0.6rem 0.75rem; }
  .data-table tr:hover td { background: rgba(61,127,255,0.04); }
  .badge { border-radius: 3px; font-family: 'Space Mono', monospace; font-size: 0.65rem; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; text-transform: uppercase; }
  .badge-open { background: rgba(0,212,255,0.12); color: #00d4ff; }
  .badge-progress { background: rgba(255,193,7,0.12); color: #ffc107; }
  .badge-done { background: rgba(0,200,83,0.12); color: #00c853; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }

  .filters-section {
    margin-top: 1rem;
    padding: 1rem;
    background: #0f121b;
    border: 1px solid #1a2035;
    border-radius: 4px;
  }
  .section-subtitle {
    font-size: 0.9rem;
    color: #4b5b75;
    text-transform: uppercase;
    margin-bottom: 1rem;
    letter-spacing: 0.05em;
  }
  .filter-group {
    margin-bottom: 1rem;
  }
  .filter-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
    color: #8b9bb4;
    margin-bottom: 0.5rem;
  }
  .filter-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .filter-row .field-input {
    flex: 1;
    background: #05070f;
    border: 1px solid #1a2035;
    color: #e2e8f0;
    padding: 0.5rem;
    border-radius: 4px;
  }
  .btn-danger {
    background: #3a1a1a;
    color: #ff6b6b;
    border: 1px solid #ff6b6b;
  }
  .btn-danger:hover {
    background: #ff6b6b;
    color: #05070f;
  }
</style>
