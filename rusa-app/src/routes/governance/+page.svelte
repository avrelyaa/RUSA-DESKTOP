<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { governanceApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount, onDestroy } from 'svelte';

  type Tab = 'votes' | 'meetings' | 'relocations' | 'events' | 'librarian';
  let activeTab = $state<Tab>('votes');

  let votes: any[] = $state([]);
  let meetings: any[] = $state([]);
  let relocations: any[] = $state([]);
  let events: any[] = $state([]);
  let redactedRecords: any[] = $state([]);
  let loading = $state(false);

  // Location dropdown (populated from backend)
  let availableLocations: string[] = $state([]);

  // Vote form
  let voteOpen = $state(false);
  let vTitle = $state('');
  let vDesc = $state('');

  // Cast vote modal
  let castOpen = $state(false);
  let selectedVote: any = $state(null);
  let castDecision = $state('');
  let castReason = $state('');

  // Meeting form
  let meetingOpen = $state(false);
  let mTitle = $state('');
  let mDesc = $state('');
  let mScheduled = $state('');
  let mLocation = $state('');

  // Relocation form
  let relocOpen = $state(false);
  let rStaff = $state('');
  let rLocation = $state('');
  let rType = $state('');
  let rStart = $state('');
  let rEnd = $state('');
  let rReason = $state('');

  // Event form
  let eventOpen = $state(false);
  let eTitle = $state('');
  let eDesc = $state('');
  let eDate = $state('');
  let eVenue = $state('');
  let eInvoice = $state('');

  // Librarian forms
  let libAction: 'redact' | 'delete' | 'restrict' = $state('redact');
  let libOpen = $state(false);
  let libTable = $state('');
  let libRecordId = $state('');
  let libReason = $state('');
  let libAccessLevel = $state('restricted');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      const tasks = [
        governanceApi.getVotes(s.token),
        governanceApi.getMeetings(s.token),
        governanceApi.getRelocations(s.token),
        governanceApi.getEvents(s.token),
        governanceApi.getRelocatableLocations(s.token),
      ];
      if (s.role_name === 'the_librarian') {
        tasks.push(governanceApi.getRedactedRecords(s.token) as any);
      }
      const results = await Promise.all(tasks);
      [votes, meetings, relocations, events, availableLocations] = results as any;
      if (s.role_name === 'the_librarian' && results[5]) {
        redactedRecords = results[5] as any[];
      }
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function initiateVote() {
    const s = $session; if (!s) return;
    if (!vTitle) { showToast('Title required', 'error'); return; }
    try {
      await governanceApi.initiateVote(s.token, vTitle, vDesc || undefined);
      showToast('Vote initiated', 'success');
      voteOpen = false; vTitle = ''; vDesc = '';
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function castVote() {
    const s = $session; if (!s || !selectedVote) return;
    if (!castDecision) { showToast('Decision required', 'error'); return; }
    if (!castReason.trim()) { showToast('A written reason is required for every vote', 'error'); return; }
    try {
      await governanceApi.castVote(s.token, selectedVote.id, castDecision, castReason);
      showToast('Vote cast', 'success'); castOpen = false; castDecision = ''; castReason = '';
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createMeeting() {
    const s = $session; if (!s) return;
    if (!mTitle) { showToast('Title required', 'error'); return; }
    if (!mScheduled) { showToast('Scheduled time required', 'error'); return; }
    try {
      await governanceApi.createMeeting(s.token, mTitle, mDesc || undefined, mScheduled, mLocation || undefined);
      showToast('Meeting created', 'success');
      meetingOpen = false; mTitle = ''; mDesc = ''; mScheduled = ''; mLocation = '';
      meetings = await governanceApi.getMeetings(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function relocate() {
    const s = $session; if (!s) return;
    if (!rStaff || !rLocation || !rType) { showToast('Staff, location, and type required', 'error'); return; }
    try {
      await governanceApi.relocateStaff(s.token, rStaff, rLocation, rType, rStart || undefined, rEnd || undefined, rReason || undefined);
      showToast('Relocation submitted', 'success');
      relocOpen = false; rStaff = ''; rLocation = ''; rType = ''; rStart = ''; rEnd = ''; rReason = '';
      relocations = await governanceApi.getRelocations(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function logEvent() {
    const s = $session; if (!s) return;
    if (!eTitle) { showToast('Title required', 'error'); return; }
    try {
      await governanceApi.logEvent(s.token, eTitle, eDesc || undefined, eDate || undefined, eVenue || undefined, eInvoice || undefined);
      showToast('Event logged', 'success');
      eventOpen = false; eTitle = ''; eDesc = ''; eDate = ''; eVenue = ''; eInvoice = '';
      events = await governanceApi.getEvents(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function performLibrarianAction() {
    const s = $session; if (!s) return;
    if (!libTable || !libRecordId) { showToast('Table and Record ID required', 'error'); return; }
    if (!libReason.trim()) { showToast('Reason is required', 'error'); return; }
    try {
      if (libAction === 'redact') {
        await governanceApi.redactRecord(s.token, libTable, libRecordId, libReason);
        showToast('Record redacted', 'success');
      } else if (libAction === 'delete') {
        if (!confirm('Are you sure you want to delete this record? This action cannot be undone.')) return;
        await governanceApi.deleteRecord(s.token, libTable, libRecordId, libReason);
        showToast('Record deleted', 'success');
      } else {
        await governanceApi.setArchivePermission(s.token, libTable, libRecordId, libAccessLevel);
        showToast('Access level updated', 'success');
      }
      libOpen = false; libTable = ''; libRecordId = ''; libReason = '';
      redactedRecords = await governanceApi.getRedactedRecords(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const voteCols = [
    { key: 'title', label: 'Motion' },
    { key: 'status', label: 'Status' },
    { key: 'scheduled_session', label: 'Session' },
    { key: 'created_at', label: 'Initiated' },
  ];
  const meetingCols = [
    { key: 'title', label: 'Meeting' },
    { key: 'scheduled_at', label: 'Scheduled' },
    { key: 'location', label: 'Location' },
  ];
  const relocCols = [
    { key: 'staff_id', label: 'Staff ID' },
    { key: 'to_location', label: 'To' },
    { key: 'relocation_type', label: 'Type' },
    { key: 'start_date', label: 'Start' },
  ];
  const eventCols = [
    { key: 'title', label: 'Event' },
    { key: 'event_date', label: 'Date' },
    { key: 'venue', label: 'Venue' },
    { key: 'venue_invoice', label: 'Invoice' },
  ];
  const redactedCols = [
    { key: 'table_name', label: 'Table' },
    { key: 'record_id', label: 'Record ID' },
    { key: 'redact_reason', label: 'Reason' },
    { key: 'redacted_at', label: 'Redacted At' },
  ];
  const decisionOpts = [
    { value: 'yay', label: 'Yay' },
    { value: 'nay', label: 'Nay' },
    { value: 'abstain', label: 'Abstain' },
  ];

  let now = $state(Date.now());
  const ticker = setInterval(() => { now = Date.now(); }, 1000);
  onDestroy(() => clearInterval(ticker));

  function getTimeRemaining(deadline: string | null): string {
    if (!deadline) return '';
    const diff = new Date(deadline).getTime() - now;
    if (diff <= 0) return 'Expired';
    const mins = Math.floor(diff / 60000);
    const secs = Math.floor((diff % 60000) / 1000);
    return `${mins}m ${secs.toString().padStart(2, '0')}s`;
  }

  function isExpired(deadline: string | null): boolean {
    if (!deadline) return false;
    return new Date(deadline).getTime() <= now;
  }

  const relocTypeOpts = [
    { value: 'temporary', label: 'Temporary' },
    { value: 'permanent', label: 'Permanent' },
    { value: 'transfer', label: 'Transfer' },
  ];

  const libTableOpts = [
    { value: 'experiments', label: 'Experiments' },
    { value: 'experiment_logs', label: 'Experiment Logs' },
    { value: 'species_archive', label: 'Species Archive' },
    { value: 'data_requests', label: 'Data Requests' },
    { value: 'incident_reports', label: 'Incident Reports' },
    { value: 'mission_reports', label: 'Mission Reports' },
    { value: 'exploration_journals', label: 'Exploration Journals' },
    { value: 'station_findings', label: 'Station Findings' },
    { value: 'security_findings', label: 'Security Findings' },
    { value: 'anomaly_reports', label: 'Anomaly Reports' },
    { value: 'budget_requests', label: 'Budget Requests' },
    { value: 'expenditure_reports', label: 'Expenditure Reports' },
    { value: 'help_requests', label: 'Help Requests' },
  ];

  const accessLevelOpts = [
    { value: 'public', label: 'Public' },
    { value: 'restricted', label: 'Restricted' },
    { value: 'classified', label: 'Classified' },
    { value: 'redacted', label: 'Redacted' },
  ];

  let locationDropOpts = $derived(availableLocations.map((l: string) => ({ value: l, label: l })));

  let isLibrarian = $derived($session?.role_name === 'the_librarian');
  let isNomadOrOverseer = $derived($session?.role_name === 'the_nomad' || $session?.role_name === 'the_overseer' || $session?.role_name === 'the_administrator');
  let isDirectorOrAbove = $derived(($session?.tier ?? 0) >= 3);
</script>

<svelte:head><title>RUSA IMS — Governance</title></svelte:head>

<PageShell title="Governance" subtitle="Votes, meetings, relocations, and event documentation">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='votes'} onclick={() => activeTab='votes'}>Votes</button>
    <button class="tab" class:active={activeTab==='meetings'} onclick={() => activeTab='meetings'}>Meetings</button>
    {#if isNomadOrOverseer}
      <button class="tab" class:active={activeTab==='relocations'} onclick={() => activeTab='relocations'}>Relocations</button>
    {/if}
    {#if $session?.role_name === 'the_coordinator'}
      <button class="tab" class:active={activeTab==='events'} onclick={() => activeTab='events'}>Events</button>
    {/if}
    {#if isLibrarian}
      <button class="tab" class:active={activeTab==='librarian'} onclick={() => activeTab='librarian'}>Librarian</button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'votes'}
    <div class="section-bar">
      <h2 class="section-title">Votes</h2>
      {#if isDirectorOrAbove}
        <button class="btn-primary" onclick={() => voteOpen = true}>+ Initiate Vote</button>
      {/if}
    </div>
    <p class="hint">Sessions scheduled at <strong>11:00</strong> and <strong>19:00</strong> (UTC ±30 min). Quorum: 8/13 Directors. Each vote requires a written reason.</p>
    {#if votes.length === 0}
      <p class="empty">No votes found</p>
    {:else}
      <div class="vote-list">
        {#each votes as vote}
          <div class="vote-card" class:expired={vote.status !== 'open' || isExpired(vote.deadline)}>
            <div class="vote-info">
              <span class="vote-title">{vote.title}</span>
              {#if vote.description}
                <span class="vote-desc">{vote.description}</span>
              {/if}
              <span class="vote-meta">Initiated: {vote.created_at ? new Date(vote.created_at).toLocaleString() : ''}{vote.scheduled_session ? ` · Session: ${vote.scheduled_session}` : ''}</span>
            </div>
            <div class="vote-right">
              <span class="status-badge status-{vote.status}">{vote.status}</span>
              {#if vote.status === 'open' && vote.deadline && !isExpired(vote.deadline)}
                <span class="countdown">⏱ {getTimeRemaining(vote.deadline)}</span>
                {#if isDirectorOrAbove}
                  <button class="btn-primary" onclick={() => { selectedVote = vote; castDecision = ''; castReason = ''; castOpen = true; }}>Cast Vote</button>
                {/if}
              {:else if vote.status === 'open' && isExpired(vote.deadline)}
                <span class="countdown expired-label">Resolving...</span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else if activeTab === 'meetings'}
    <div class="section-bar">
      <h2 class="section-title">Meetings</h2>
      {#if isDirectorOrAbove}
        <button class="btn-primary" onclick={() => meetingOpen = true}>+ Create Meeting</button>
      {/if}
    </div>
    <Table columns={meetingCols} rows={meetings} />
  {:else if activeTab === 'relocations'}
    <div class="section-bar">
      <h2 class="section-title">Staff Relocations</h2>
      <button class="btn-primary" onclick={() => relocOpen = true}>+ Relocate Staff</button>
    </div>
    <Table columns={relocCols} rows={relocations} />
  {:else if activeTab === 'events'}
    <div class="section-bar">
      <h2 class="section-title">Event Documents</h2>
      <button class="btn-primary" onclick={() => eventOpen = true}>+ Log Event</button>
    </div>
    <Table columns={eventCols} rows={events} />
  {:else if activeTab === 'librarian'}
    <div class="section-bar">
      <h2 class="section-title">Librarian — Archive Management</h2>
      <div class="lib-action-bar">
        <button class="btn-lib" class:active-lib={libAction==='redact'} onclick={() => { libAction='redact'; libOpen=true; }}>⬛ Redact Record</button>
        <button class="btn-lib btn-lib-danger" class:active-lib={libAction==='delete'} onclick={() => { libAction='delete'; libOpen=true; }}>🗑 Delete Record</button>
        <button class="btn-lib" class:active-lib={libAction==='restrict'} onclick={() => { libAction='restrict'; libOpen=true; }}>🔒 Restrict Access</button>
      </div>
    </div>
    <p class="hint">Voting is for evaluating requests — not for creating new documents. Use the actions above to manage existing records.</p>
    <h3 class="section-subtitle">Redaction Log</h3>
    <Table columns={redactedCols} rows={redactedRecords} />
  {/if}
</PageShell>

<Modal bind:open={voteOpen} title="Initiate Vote">
  <div class="form">
    <Field label="Motion Title" bind:value={vTitle} required />
    <Field label="Description" type="textarea" bind:value={vDesc} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => voteOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={initiateVote}>Initiate</button>
    </div>
  </div>
</Modal>

<Modal bind:open={castOpen} title="Cast Vote: {selectedVote?.title}">
  <div class="form">
    {#if selectedVote?.deadline}
      <p class="vote-timer">Time remaining: <span class="countdown">{getTimeRemaining(selectedVote.deadline)}</span></p>
    {/if}
    <p class="hint">Voting windows: <strong>11:00</strong> and <strong>19:00</strong> UTC (±30 min). Options: Yay / Nay / Abstain. A written reason is mandatory.</p>
    <Field label="Your Vote" type="select" bind:value={castDecision} options={decisionOpts} required />
    <Field label="Reason (required)" type="textarea" bind:value={castReason} rows={3} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => castOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={castVote}>Cast Vote</button>
    </div>
  </div>
</Modal>

<Modal bind:open={meetingOpen} title="Create Meeting">
  <div class="form">
    <Field label="Title" bind:value={mTitle} required />
    <Field label="Description" type="textarea" bind:value={mDesc} />
    <Field label="Scheduled At" type="datetime-local" bind:value={mScheduled} required />
    <Field label="Location" bind:value={mLocation} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => meetingOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createMeeting}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={relocOpen} title="Relocate Staff">
  <div class="form">
    <Field label="Staff User ID" bind:value={rStaff} required />
    {#if locationDropOpts.length > 0}
      <Field label="Destination" type="select" bind:value={rLocation} options={locationDropOpts} required />
    {:else}
      <Field label="Destination" bind:value={rLocation} required />
    {/if}
    <Field label="Type" type="select" bind:value={rType} options={relocTypeOpts} required />
    <Field label="Start Date" type="date" bind:value={rStart} />
    <Field label="End Date" type="date" bind:value={rEnd} />
    <Field label="Reason" type="textarea" bind:value={rReason} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => relocOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={relocate}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={eventOpen} title="Log Event Document">
  <div class="form">
    <Field label="Title" bind:value={eTitle} required />
    <Field label="Description" type="textarea" bind:value={eDesc} />
    <Field label="Event Date" type="date" bind:value={eDate} />
    <Field label="Venue" bind:value={eVenue} />
    <Field label="Venue Invoice / Reference" bind:value={eInvoice} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => eventOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={logEvent}>Log</button>
    </div>
  </div>
</Modal>

<Modal bind:open={libOpen} title={libAction === 'redact' ? 'Redact Record' : libAction === 'delete' ? 'Delete Record' : 'Restrict Access'}>
  <div class="form">
    {#if libAction === 'delete'}
      <div class="warning-box">⚠ Deleting a record is irreversible and will be permanently logged. Proceed only if absolutely necessary.</div>
    {/if}
    <Field label="Table Name" type="select" bind:value={libTable} options={libTableOpts} required />
    <Field label="Record ID (UUID)" bind:value={libRecordId} required />
    {#if libAction === 'restrict'}
      <Field label="Access Level" type="select" bind:value={libAccessLevel} options={accessLevelOpts} required />
    {/if}
    <Field label="Reason (required)" type="textarea" bind:value={libReason} rows={3} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => libOpen = false}>Cancel</button>
      {#if libAction === 'delete'}
        <button class="btn-danger" onclick={performLibrarianAction}>Delete</button>
      {:else}
        <button class="btn-primary" onclick={performLibrarianAction}>Confirm</button>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; margin-bottom: 1.5rem; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .section-subtitle { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin: 1.25rem 0 0.5rem; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .hint { font-size: 0.78rem; color: #6a7f9a; margin: 0 0 1rem; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-lib { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.78rem; padding: 0.5rem 1rem; transition: all 0.15s; }
  .btn-lib:hover, .btn-lib.active-lib { border-color: #3d7fff; color: #3d7fff; }
  .btn-lib-danger { }
  .btn-lib-danger:hover, .btn-lib-danger.active-lib { border-color: #ff4466; color: #ff4466; }
  .lib-action-bar { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .vote-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .vote-card { display: flex; align-items: flex-start; justify-content: space-between; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; gap: 1rem; }
  .vote-card.expired { opacity: 0.7; }
  .vote-info { display: flex; flex-direction: column; gap: 0.25rem; flex: 1; }
  .vote-title { font-size: 0.95rem; color: #e8eeff; font-weight: 500; }
  .vote-desc { font-size: 0.82rem; color: #6a7f9a; }
  .vote-meta { font-size: 0.75rem; color: #4a5d82; }
  .vote-right { display: flex; flex-direction: column; align-items: flex-end; gap: 0.5rem; flex-shrink: 0; }
  .countdown { font-family: 'Space Mono', monospace; font-size: 0.8rem; color: #ffaa00; }
  .expired-label { color: #ff4466; }
  .status-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .status-open { background: rgba(0,212,255,0.15); color: #00d4ff; }
  .status-passed { background: rgba(0,255,128,0.12); color: #00ff80; }
  .status-failed { background: rgba(255,68,102,0.12); color: #ff4466; }
  .status-interrupted { background: rgba(255,170,0,0.12); color: #ffaa00; }
  .empty { color: #4a5d82; padding: 1.5rem 0; font-size: 0.9rem; }
  .vote-timer { font-size: 0.82rem; color: #8fa3cc; margin: 0; }
  .warning-box { background: rgba(255,68,102,0.1); border: 1px solid rgba(255,68,102,0.3); border-radius: 4px; color: #ff4466; font-size: 0.82rem; padding: 0.75rem; }
</style>

