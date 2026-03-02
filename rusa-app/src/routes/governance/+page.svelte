<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { governanceApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'votes' | 'meetings' | 'relocations' | 'events';
  let activeTab = $state<Tab>('votes');

  let votes: any[] = $state([]);
  let meetings: any[] = $state([]);
  let relocations: any[] = $state([]);
  let events: any[] = $state([]);
  let loading = $state(false);

  // Vote form
  let voteOpen = $state(false);
  let vTitle = $state('');
  let vDesc = $state('');

  // Cast vote modal
  let castOpen = $state(false);
  let selectedVote: any = $state(null);
  let castDecision = $state('');

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

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [votes, meetings, relocations, events] = await Promise.all([
        governanceApi.getVotes(s.token),
        governanceApi.getMeetings(s.token),
        governanceApi.getRelocations(s.token),
        governanceApi.getEvents(s.token),
      ]);
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
    try {
      await governanceApi.castVote(s.token, selectedVote.id, castDecision);
      showToast('Vote cast', 'success'); castOpen = false;
      votes = await governanceApi.getVotes(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createMeeting() {
    const s = $session; if (!s) return;
    if (!mTitle) { showToast('Title required', 'error'); return; }
    try {
      await governanceApi.createMeeting(s.token, mTitle, mDesc || undefined, mScheduled || undefined, mLocation || undefined);
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
      await governanceApi.logEvent(s.token, eTitle, eDesc || undefined, eDate || undefined, eVenue || undefined);
      showToast('Event logged', 'success');
      eventOpen = false; eTitle = ''; eDesc = ''; eDate = ''; eVenue = '';
      events = await governanceApi.getEvents(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const voteCols = [
    { key: 'title', label: 'Motion' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Initiated' },
  ];
  const meetingCols = [
    { key: 'title', label: 'Meeting' },
    { key: 'scheduled_at', label: 'Scheduled' },
    { key: 'location', label: 'Location' },
  ];
  const relocCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'to_location', label: 'To' },
    { key: 'relocation_type', label: 'Type' },
    { key: 'start_date', label: 'Start' },
  ];
  const eventCols = [
    { key: 'title', label: 'Event' },
    { key: 'event_date', label: 'Date' },
    { key: 'venue', label: 'Venue' },
  ];
  const decisionOpts = [
    { value: 'yes', label: 'Yes' },
    { value: 'no', label: 'No' },
    { value: 'abstain', label: 'Abstain' },
  ];
  const relocTypeOpts = [
    { value: 'temporary', label: 'Temporary' },
    { value: 'permanent', label: 'Permanent' },
    { value: 'transfer', label: 'Transfer' },
  ];
</script>

<svelte:head><title>RUSA IMS — Governance</title></svelte:head>

<PageShell title="Governance" subtitle="Votes, meetings, relocations, and event documentation">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='votes'} onclick={() => activeTab='votes'}>Votes</button>
    <button class="tab" class:active={activeTab==='meetings'} onclick={() => activeTab='meetings'}>Meetings</button>
    <button class="tab" class:active={activeTab==='relocations'} onclick={() => activeTab='relocations'}>Relocations</button>
    <button class="tab" class:active={activeTab==='events'} onclick={() => activeTab='events'}>Events</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'votes'}
    <div class="section-bar">
      <h2 class="section-title">Active Votes</h2>
      <button class="btn-primary" onclick={() => voteOpen = true}>+ Initiate Vote</button>
    </div>
    <Table columns={voteCols} rows={votes} onRowClick={(r) => { selectedVote = r; castDecision = ''; castOpen = true; }} />
  {:else if activeTab === 'meetings'}
    <div class="section-bar">
      <h2 class="section-title">Meetings</h2>
      <button class="btn-primary" onclick={() => meetingOpen = true}>+ Create Meeting</button>
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
    <Field label="Your Decision" type="select" bind:value={castDecision} options={decisionOpts} required />
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
    <Field label="Scheduled At" type="datetime-local" bind:value={mScheduled} />
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
    <Field label="Destination" bind:value={rLocation} required />
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

<Modal bind:open={eventOpen} title="Log Event">
  <div class="form">
    <Field label="Title" bind:value={eTitle} required />
    <Field label="Description" type="textarea" bind:value={eDesc} />
    <Field label="Event Date" type="date" bind:value={eDate} />
    <Field label="Venue" bind:value={eVenue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => eventOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={logEvent}>Log</button>
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
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
