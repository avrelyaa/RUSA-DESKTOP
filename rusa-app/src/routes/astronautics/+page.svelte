<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { canPerform } from '$lib/stores/permissions';
  import { astronautApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'missions' | 'journals' | 'sectors' | 'planets' | 'conclusion_requests';
  type JournalView = 'my_logbook' | 'wanderer_directory';

  let activeTab = $state<Tab>('missions');
  let journalView = $state<JournalView>('my_logbook');

  let missions: any[] = $state([]);
  let journals: any[] = $state([]);
  let sectors: any[] = $state([]);
  let planets: any[] = $state([]);
  let ships: any[] = $state([]);
  let conclusionRequests: any[] = $state([]);
  let directoryRows: any[] = $state([]);
  let allUsers: any[] = $state([]);
  let myStats: any = $state(null);
  let loading = $state(false);

  // Mission form
  let missionOpen = $state(false);
  let mTitle = $state('');
  let mDesc = $state('');
  let mType = $state('');
  let mShip = $state('');
  let mSector = $state('');
  let mStart = $state('');

  // Journal form
  let journalOpen = $state(false);
  let jMissionId = $state('');
  let jMissionLocation = $state('');
  let jMissionType = $state<'terraneous' | 'interstellar'>('terraneous');
  let jProgressReport = $state('');
  let jNextSteps = $state('');
  let jApprovalStatus = $state<'draft' | 'submitted'>('draft');

  // Directory filters + details
  let directoryQuery = $state('');
  let directoryFilter = $state<'all' | 'assigned' | 'waiting' | 'official' | 'available'>('all');
  let crewInfoOpen = $state(false);
  let selectedCrew: any = $state(null);

  // Sector form
  let sectorOpen = $state(false);
  let secName = $state('');
  let secDesc = $state('');

  // Planet form
  let planetOpen = $state(false);
  let planetName = $state('');
  let planetDesc = $state('');
  let planetStar = $state('');
  // Rename planet
  let renamePlanetOpen = $state(false);
  let renamePlanetId = $state('');
  let renamePlanetName = $state('');

  // Conclusion request form (astronaut)
  let conclusionOpen = $state(false);
  let cMissionId = $state('');
  let cSummary = $state('');

  // Review form (wanderer/taskmaster)
  let reviewOpen = $state(false);
  let reviewTarget: any = $state(null);
  let reviewRequestId = $state('');
  let reviewDecision = $state('approved');
  let reviewNotes = $state('');

  const isWanderer = $derived(canPerform($session, 'the_wanderer'));
  const isDirector = $derived(
    canPerform($session, 'the_wanderer') || canPerform($session, 'the_taskmaster')
  );

  const missionLinkOptions = $derived([
    { value: '', label: 'Not linked to a mission' },
    ...missions.map((m: any) => ({ value: m.id, label: m.title }))
  ]);

  const reviewMission = $derived(
    reviewTarget ? missions.find((m: any) => m.id === reviewTarget.mission_id) : null
  );

  const isAuthorizedForConclusionReview = $derived(
    reviewMission ? reviewMission.created_by === $session?.user_id : false
  );

  function normalizeStatus(status: string | undefined | null): 'draft' | 'submitted' | 'approved' {
    if (status === 'submitted') return 'submitted';
    if (status === 'approved') return 'approved';
    return 'draft';
  }

  function formatDate(value: string | undefined | null): string {
    if (!value) return 'Unknown time';
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return 'Unknown time';
    return date.toLocaleString();
  }

  function getUserName(userId: string | undefined | null): string {
    if (!userId) return '-';
    const u = allUsers.find((user: any) => user.id === userId);
    return u ? (u.full_name ?? u.username ?? userId) : userId;
  }

  function getEntryLocation(entry: any): string {
    return entry.mission_location || entry.title || 'Unknown location';
  }

  function getEntryType(entry: any): 'terraneous' | 'interstellar' {
    return entry.mission_type === 'interstellar' ? 'interstellar' : 'terraneous';
  }

  function getEntryProgress(entry: any): string {
    return entry.progress_report || entry.content || 'No progress report provided yet.';
  }

  function getEntryNextSteps(entry: any): string {
    return entry.next_steps || 'No next steps recorded yet.';
  }

  function directoryStatusLabel(status: string | undefined | null): string {
    const normalized = normalizeStatus(status);
    if (normalized === 'submitted') return 'Assigned - Waiting on The Wanderer';
    if (normalized === 'approved') return 'Archived / Official';
    if (normalized === 'draft') return 'Assigned - Drafting Report';
    return 'Available';
  }

  function isAssignedDirectoryStatus(status: string | undefined | null): boolean {
    const normalized = normalizeStatus(status);
    return normalized === 'draft' || normalized === 'submitted';
  }

  const sortedJournals = $derived(
    [...journals].sort((a: any, b: any) => {
      const left = Date.parse(a.updated_at || a.created_at || 0);
      const right = Date.parse(b.updated_at || b.created_at || 0);
      return right - left;
    })
  );

  const activeLogEntries = $derived(
    sortedJournals.filter((entry: any) => normalizeStatus(entry.approval_status) === 'draft')
  );

  const pendingLogEntries = $derived(
    sortedJournals.filter((entry: any) => normalizeStatus(entry.approval_status) === 'submitted')
  );

  const approvedLogEntries = $derived(
    sortedJournals.filter((entry: any) => normalizeStatus(entry.approval_status) === 'approved')
  );

  const terrainMissions = $derived(
    myStats?.terrain_missions ??
      approvedLogEntries.filter((entry: any) => getEntryType(entry) === 'terraneous').length
  );

  const interstellarMissions = $derived(
    myStats?.interstellar_missions ??
      approvedLogEntries.filter((entry: any) => getEntryType(entry) === 'interstellar').length
  );

  const filteredDirectoryRows = $derived(
    directoryRows.filter((row: any) => {
      const q = directoryQuery.trim().toLowerCase();
      const searchable = `${row.full_name ?? ''} ${row.username ?? ''} ${row.current_assignment ?? ''}`.toLowerCase();
      const queryPass = !q || searchable.includes(q);

      const normalized = normalizeStatus(row.current_status);
      const filterPass =
        directoryFilter === 'all' ||
        (directoryFilter === 'assigned' && (normalized === 'draft' || normalized === 'submitted')) ||
        (directoryFilter === 'waiting' && normalized === 'submitted') ||
        (directoryFilter === 'official' && normalized === 'approved') ||
        (directoryFilter === 'available' && normalized === 'draft' && !row.current_assignment);

      return queryPass && filterPass;
    })
  );

  onMount(async () => {
    const s = $session;
    if (!s) return;

    loading = true;
    try {
      [missions, journals, sectors, planets, ships, conclusionRequests, allUsers, myStats, directoryRows] =
        await Promise.all([
          astronautApi.getMissions(s.token),
          astronautApi.getJournals(s.token),
          astronautApi.getSectors(s.token),
          astronautApi.getPlanets(s.token),
          astronautApi.getShips(s.token),
          astronautApi.getConclusionRequests(s.token),
          userApi.getAll(s.token),
          astronautApi.getStats(s.token),
          astronautApi.getAstronautDirectory(s.token)
        ]);
    } catch (e: any) {
      showToast('Failed to load astronautics data: ' + e, 'error');
    }
    loading = false;
  });

  async function createMission() {
    const s = $session;
    if (!s) return;
    if (!mTitle) {
      showToast('Title required', 'error');
      return;
    }
    try {
      await astronautApi.createMission(
        s.token,
        mTitle,
        mDesc || undefined,
        mType || undefined,
        mShip || undefined,
        mSector || undefined,
        undefined,
        mStart ? mStart + 'T00:00:00Z' : undefined
      );
      showToast('Mission created', 'success');
      missionOpen = false;
      mTitle = '';
      mDesc = '';
      mType = '';
      mShip = '';
      mSector = '';
      mStart = '';
      missions = await astronautApi.getMissions(s.token);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  async function createJournal() {
    const s = $session;
    if (!s) return;

    if (!jMissionLocation.trim() || !jProgressReport.trim() || !jNextSteps.trim()) {
      showToast('Mission location, progress report, and next steps are required', 'error');
      return;
    }

    try {
      await astronautApi.createJournal(
        s.token,
        jMissionLocation.trim(),
        jMissionType,
        jProgressReport.trim(),
        jNextSteps.trim(),
        jApprovalStatus,
        jMissionId || undefined
      );

      showToast('Logbook entry saved', 'success');
      journalOpen = false;
      jMissionId = '';
      jMissionLocation = '';
      jMissionType = 'terraneous';
      jProgressReport = '';
      jNextSteps = '';
      jApprovalStatus = 'draft';

      [journals, myStats, directoryRows] = await Promise.all([
        astronautApi.getJournals(s.token),
        astronautApi.getStats(s.token),
        astronautApi.getAstronautDirectory(s.token)
      ]);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  async function createSector() {
    const s = $session;
    if (!s) return;
    if (!secName) {
      showToast('Name required', 'error');
      return;
    }
    try {
      await astronautApi.createSector(s.token, secName, secDesc || undefined);
      showToast('Sector created', 'success');
      sectorOpen = false;
      secName = '';
      secDesc = '';
      sectors = await astronautApi.getSectors(s.token);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  async function createPlanet() {
    const s = $session;
    if (!s) return;
    if (!planetName) {
      showToast('Name required', 'error');
      return;
    }
    try {
      await astronautApi.createPlanet(s.token, planetName, planetDesc || undefined, planetStar || undefined);
      showToast('Planet registered', 'success');
      planetOpen = false;
      planetName = '';
      planetDesc = '';
      planetStar = '';
      planets = await astronautApi.getPlanets(s.token);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  function openRenamePlanet(row: any) {
    renamePlanetId = row.id;
    renamePlanetName = row.name;
    renamePlanetOpen = true;
  }

  async function submitRenamePlanet() {
    const s = $session;
    if (!s) return;
    if (!renamePlanetName) {
      showToast('Name required', 'error');
      return;
    }
    try {
      await astronautApi.renamePlanet(s.token, renamePlanetId, renamePlanetName);
      showToast('Planet renamed', 'success');
      renamePlanetOpen = false;
      planets = await astronautApi.getPlanets(s.token);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  async function submitConclusionRequest() {
    const s = $session;
    if (!s) return;
    if (!cMissionId || !cSummary) {
      showToast('Mission ID and report summary required', 'error');
      return;
    }
    try {
      await astronautApi.submitConclusionRequest(s.token, cMissionId, cSummary);
      showToast('Conclusion request submitted', 'success');
      conclusionOpen = false;
      cMissionId = '';
      cSummary = '';
      conclusionRequests = await astronautApi.getConclusionRequests(s.token);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  function openReview(row: any) {
    reviewTarget = row;
    reviewRequestId = row.id;
    reviewDecision = 'approved';
    reviewNotes = '';
    reviewOpen = true;
  }

  async function submitReview() {
    const s = $session;
    if (!s) return;
    try {
      await astronautApi.reviewConclusionRequest(
        s.token,
        reviewRequestId,
        reviewDecision,
        reviewNotes || undefined
      );
      showToast('Review submitted', 'success');
      reviewOpen = false;

      [conclusionRequests, journals, myStats, directoryRows] = await Promise.all([
        astronautApi.getConclusionRequests(s.token),
        astronautApi.getJournals(s.token),
        astronautApi.getStats(s.token),
        astronautApi.getAstronautDirectory(s.token)
      ]);
    } catch (e: any) {
      showToast('Failed: ' + e, 'error');
    }
  }

  const missionCols = [
    { key: 'title', label: 'Mission' },
    { key: 'mission_type', label: 'Type' },
    { key: 'status', label: 'Status' },
    { key: 'start_date', label: 'Start Date' }
  ];

  const sectorCols = [
    { key: 'name', label: 'Sector Name' },
    { key: 'description', label: 'Description' }
  ];

  const planetCols = [
    { key: 'name', label: 'Planet Name' },
    { key: 'description', label: 'Description' },
    { key: 'star_system', label: 'Star System' }
  ];

  const conclusionCols = [
    { key: 'mission_id', label: 'Mission ID' },
    { key: 'report_summary', label: 'Report Summary' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Submitted' }
  ];
</script>

<svelte:head><title>RUSA IMS - Astronautics</title></svelte:head>

<PageShell title="Astronautics" subtitle="Mission operations, private logbooks, and crew directory">
  <div class="tabs">
    <button class="tab" class:active={activeTab === 'missions'} onclick={() => (activeTab = 'missions')}>Missions</button>
    <button class="tab" class:active={activeTab === 'journals'} onclick={() => (activeTab = 'journals')}>Journals</button>
    <button class="tab" class:active={activeTab === 'sectors'} onclick={() => (activeTab = 'sectors')}>Sectors</button>
    <button class="tab" class:active={activeTab === 'planets'} onclick={() => (activeTab = 'planets')}>Planets</button>
    <button class="tab" class:active={activeTab === 'conclusion_requests'} onclick={() => (activeTab = 'conclusion_requests')}>Conclusion Requests</button>
  </div>

  {#if loading}
    <p class="loading">Loading astronautics data...</p>
  {:else if activeTab === 'missions'}
    <div class="section-bar">
      <h2 class="section-title">Missions</h2>
      <button class="btn-primary" onclick={() => (missionOpen = true)}>+ Create Mission</button>
    </div>
    <Table columns={missionCols} rows={missions} />
  {:else if activeTab === 'journals'}
    <section class="journal-hub">
      <div class="hub-header">
        <div>
          <h2 class="hub-title">Astronaut / Wanderer Hub</h2>
          <p class="hub-subtitle">Private log writing and public-safe crew experience directory</p>
        </div>
        <div class="hub-view-tabs">
          <button class="hub-view-tab" class:active={journalView === 'my_logbook'} onclick={() => (journalView = 'my_logbook')}>My Logbook</button>
          <button class="hub-view-tab" class:active={journalView === 'wanderer_directory'} onclick={() => (journalView = 'wanderer_directory')}>Wanderer Directory</button>
        </div>
      </div>

      {#if journalView === 'my_logbook'}
        <div class="stats-banner">
          <div class="stat-card terrain">
            <p class="stat-label">Terrain Missions</p>
            <p class="stat-value">{terrainMissions}</p>
          </div>
          <div class="stat-card interstellar">
            <p class="stat-label">Interstellar Missions</p>
            <p class="stat-value">{interstellarMissions}</p>
          </div>
          <div class="stat-card waiting">
            <p class="stat-label">Waiting Approval</p>
            <p class="stat-value">{pendingLogEntries.length}</p>
          </div>
        </div>

        <div class="logbook-action-row">
          <div>
            <h3 class="timeline-title">Captain's Log Timeline</h3>
            <p class="timeline-subtitle">One personal mission logbook with structured progress reports</p>
          </div>
          <button class="btn-primary" onclick={() => (journalOpen = true)}>Add Progress Report</button>
        </div>

        <div class="timeline-grid">
          <section class="timeline-column">
            <div class="timeline-column-head">
              <h4>Active Missions</h4>
              <span class="count-chip">{activeLogEntries.length}</span>
            </div>
            {#if activeLogEntries.length === 0}
              <p class="timeline-empty">No active draft entries yet.</p>
            {:else}
              {#each activeLogEntries as entry (entry.id)}
                <article class="log-card active">
                  <div class="log-card-top">
                    <p class="log-location">{getEntryLocation(entry)}</p>
                    <span class="log-type" class:interstellar={getEntryType(entry) === 'interstellar'}>
                      {getEntryType(entry) === 'interstellar' ? 'Interstellar' : 'Terraneous'}
                    </span>
                  </div>
                  <p class="log-meta">Draft entry • {formatDate(entry.updated_at || entry.created_at)}</p>
                  <p class="log-block-title">Progress Report</p>
                  <p class="log-block-text">{getEntryProgress(entry)}</p>
                  <p class="log-block-title">Next Steps</p>
                  <p class="log-block-text">{getEntryNextSteps(entry)}</p>
                </article>
              {/each}
            {/if}
          </section>

          <section class="timeline-column">
            <div class="timeline-column-head">
              <h4>Pending Approval</h4>
              <span class="count-chip">{pendingLogEntries.length}</span>
            </div>
            {#if pendingLogEntries.length === 0}
              <p class="timeline-empty">No entries waiting for The Wanderer.</p>
            {:else}
              {#each pendingLogEntries as entry (entry.id)}
                <article class="log-card pending">
                  <div class="log-card-top">
                    <p class="log-location">{getEntryLocation(entry)}</p>
                    <span class="badge-waiting">Waiting on The Wanderer</span>
                  </div>
                  <p class="log-meta">Submitted • {formatDate(entry.updated_at || entry.created_at)}</p>
                  <p class="log-block-title">Progress Report</p>
                  <p class="log-block-text">{getEntryProgress(entry)}</p>
                  <p class="log-block-title">Next Steps</p>
                  <p class="log-block-text">{getEntryNextSteps(entry)}</p>
                </article>
              {/each}
            {/if}
          </section>

          <section class="timeline-column">
            <div class="timeline-column-head">
              <h4>Archived / Official</h4>
              <span class="count-chip">{approvedLogEntries.length}</span>
            </div>
            {#if approvedLogEntries.length === 0}
              <p class="timeline-empty">No approved entries yet.</p>
            {:else}
              {#each approvedLogEntries as entry (entry.id)}
                <article class="log-card archived">
                  <div class="log-card-top">
                    <p class="log-location">{getEntryLocation(entry)}</p>
                    <span class="badge-official">Official</span>
                  </div>
                  <p class="log-meta">Approved • {formatDate(entry.updated_at || entry.created_at)}</p>
                  <p class="log-block-title">Progress Report</p>
                  <p class="log-block-text">{getEntryProgress(entry)}</p>
                  <p class="log-block-title">Next Steps</p>
                  <p class="log-block-text">{getEntryNextSteps(entry)}</p>
                </article>
              {/each}
            {/if}
          </section>
        </div>
      {:else}
        <div class="directory-panel">
          <div class="directory-toolbar">
            <div>
              <h3 class="timeline-title">Wanderer Directory</h3>
              <p class="timeline-subtitle">Crew manifest with assignment metadata and approved mission counters only</p>
            </div>
            <div class="directory-filters">
              <input class="directory-search" bind:value={directoryQuery} placeholder="Search astronaut or assignment" />
              <select class="directory-select" bind:value={directoryFilter}>
                <option value="all">All Statuses</option>
                <option value="assigned">Assigned</option>
                <option value="waiting">Waiting Approval</option>
                <option value="official">Official Logs</option>
                <option value="available">Available</option>
              </select>
            </div>
          </div>

          <div class="manifest-grid">
            {#if filteredDirectoryRows.length === 0}
              <p class="timeline-empty">No crew matches the current filters.</p>
            {:else}
              {#each filteredDirectoryRows as crew (crew.astronaut_id)}
                <button class="crew-card" onclick={() => { selectedCrew = crew; crewInfoOpen = true; }}>
                  <p class="crew-name">{crew.full_name}</p>
                  <p class="crew-status">Current Status: {directoryStatusLabel(crew.current_status)}</p>
                  <p class="crew-assignment">
                    {isAssignedDirectoryStatus(crew.current_status)
                      ? `Assignment: ${crew.current_assignment || 'Classified Assignment'}`
                      : 'Assignment: Available'}
                  </p>
                  <div class="crew-badges">
                    <span class="crew-badge terrain">Terrain: {crew.terrain_count ?? 0}</span>
                    <span class="crew-badge interstellar">Interstellar: {crew.interstellar_count ?? 0}</span>
                  </div>
                </button>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    </section>
  {:else if activeTab === 'sectors'}
    <div class="section-bar">
      <h2 class="section-title">Sectors</h2>
      {#if isWanderer}
        <button class="btn-primary" onclick={() => (sectorOpen = true)}>+ Create Sector</button>
      {/if}
    </div>
    <Table columns={sectorCols} rows={sectors} />
  {:else if activeTab === 'planets'}
    <div class="section-bar">
      <h2 class="section-title">Planets</h2>
      {#if isWanderer}
        <button class="btn-primary" onclick={() => (planetOpen = true)}>+ Register Planet</button>
      {/if}
    </div>
    <Table columns={planetCols} rows={planets}>
      {#snippet rowActions(row)}
        {#if isWanderer}
          <button class="btn-action" onclick={() => openRenamePlanet(row)}>Rename</button>
        {/if}
      {/snippet}
    </Table>
  {:else if activeTab === 'conclusion_requests'}
    <div class="section-bar">
      <h2 class="section-title">Conclusion Requests</h2>
      {#if !isDirector}
        <button class="btn-primary" onclick={() => (conclusionOpen = true)}>+ Request Conclusion</button>
      {/if}
    </div>
    <Table columns={conclusionCols} rows={conclusionRequests}>
      {#snippet rowActions(row)}
        {#if isDirector && row.status === 'pending'}
          <button class="btn-action" onclick={() => openReview(row)}>Review</button>
        {/if}
      {/snippet}
    </Table>
  {/if}
</PageShell>

<Modal bind:open={missionOpen} title="Create Mission">
  <div class="form">
    <Field label="Title" bind:value={mTitle} required />
    <Field label="Description" type="textarea" bind:value={mDesc} />
    <Field label="Mission Type" bind:value={mType} placeholder="e.g. exploration, supply" />
    <Field label="Ship ID" bind:value={mShip} />
    <Field label="Sector ID" bind:value={mSector} />
    <Field label="Start Date" type="date" bind:value={mStart} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (missionOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={createMission}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={journalOpen} title="Add Progress Report">
  <div class="form">
    <Field label="Linked Mission (Optional)" type="select" bind:value={jMissionId} options={missionLinkOptions} />
    <Field label="Mission Location" bind:value={jMissionLocation} placeholder="Planet or Sector" required />
    <Field
      label="Mission Type"
      type="select"
      bind:value={jMissionType}
      options={[
        { value: 'terraneous', label: 'Terraneous' },
        { value: 'interstellar', label: 'Interstellar' }
      ]}
    />
    <Field label="Progress Report" type="textarea" bind:value={jProgressReport} rows={6} required />
    <Field label="Next Steps" type="textarea" bind:value={jNextSteps} rows={4} required />
    <Field
      label="Approval Status"
      type="select"
      bind:value={jApprovalStatus}
      options={[
        { value: 'draft', label: 'Draft' },
        { value: 'submitted', label: 'Submitted for Approval' }
      ]}
    />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (journalOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={createJournal}>Save Entry</button>
    </div>
  </div>
</Modal>

<Modal bind:open={sectorOpen} title="Create Sector">
  <div class="form">
    <Field label="Sector Name" bind:value={secName} required />
    <Field label="Description" type="textarea" bind:value={secDesc} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (sectorOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={createSector}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={planetOpen} title="Register Planet">
  <div class="form">
    <Field label="Planet Name" bind:value={planetName} required />
    <Field label="Description" type="textarea" bind:value={planetDesc} />
    <Field label="Star System" bind:value={planetStar} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (planetOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={createPlanet}>Register</button>
    </div>
  </div>
</Modal>

<Modal bind:open={renamePlanetOpen} title="Rename Planet">
  <div class="form">
    <Field label="New Name" bind:value={renamePlanetName} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (renamePlanetOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={submitRenamePlanet}>Rename</button>
    </div>
  </div>
</Modal>

<Modal bind:open={conclusionOpen} title="Request Mission Conclusion">
  <div class="form">
    <Field label="Mission ID" bind:value={cMissionId} required />
    <Field label="Report Summary" type="textarea" bind:value={cSummary} rows={5} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (conclusionOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={submitConclusionRequest}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={reviewOpen} title="Review Conclusion Request">
  <div class="form">
    {#if reviewMission}
      <div class="info-block">
        <p class="info-text">Mission: <strong>{reviewMission.title}</strong></p>
        <p class="info-text">Mission Creator: <strong>{getUserName(reviewMission.created_by)}</strong></p>
        {#if isAuthorizedForConclusionReview}
          <p class="auth-badge auth-ok">You are authorized to review this conclusion request.</p>
        {:else}
          <p class="auth-badge auth-denied">Not authorized. Only the mission creator can review this request.</p>
        {/if}
      </div>
    {/if}
    <Field
      label="Decision"
      type="select"
      bind:value={reviewDecision}
      options={[
        { value: 'approved', label: 'Approve' },
        { value: 'rejected', label: 'Reject' }
      ]}
    />
    <Field label="Review Notes" type="textarea" bind:value={reviewNotes} rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => (reviewOpen = false)}>Cancel</button>
      <button class="btn-primary" onclick={submitReview} disabled={!isAuthorizedForConclusionReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={crewInfoOpen} title="Crew Manifest Record">
  {#if selectedCrew}
    <div class="crew-modal">
      <p class="crew-modal-name">{selectedCrew.full_name}</p>
      <p class="crew-modal-status">{directoryStatusLabel(selectedCrew.current_status)}</p>
      <p class="crew-modal-line">Comm-link: {selectedCrew.comm_link || 'Unavailable'}</p>
      <p class="crew-modal-line">
        Latest Assignment: {selectedCrew.current_assignment || 'No active assignment'}
      </p>
      <div class="crew-modal-badges">
        <span class="crew-badge terrain">Terrain: {selectedCrew.terrain_count ?? 0}</span>
        <span class="crew-badge interstellar">Interstellar: {selectedCrew.interstellar_count ?? 0}</span>
      </div>
      <div class="crew-modal-visits">
        <p class="log-block-title">Visited Planets / Sectors</p>
        {#if selectedCrew.visited_locations?.length}
          <ul>
            {#each selectedCrew.visited_locations as location}
              <li>{location}</li>
            {/each}
          </ul>
        {:else}
          <p class="timeline-empty">No unclassified location history available yet.</p>
        {/if}
      </div>
    </div>
  {/if}
</Modal>

<style>
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid #2e1f43;
    margin-bottom: 1.5rem;
  }

  .tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: #b7a6cc;
    cursor: pointer;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    padding: 0.75rem 1.25rem;
    text-transform: uppercase;
    transition: all 0.15s;
  }

  .tab.active {
    color: #ffdf8f;
    border-bottom-color: #ffdf8f;
  }

  .loading {
    color: #9f8ab7;
    padding: 2rem 0;
  }

  .section-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .section-title {
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #cdb5e6;
  }

  .journal-hub {
    border: 1px solid #40205b;
    border-radius: 14px;
    background:
      radial-gradient(1200px 400px at -10% -30%, rgba(227, 188, 255, 0.2), transparent 60%),
      radial-gradient(900px 500px at 120% 120%, rgba(91, 199, 255, 0.15), transparent 60%),
      linear-gradient(165deg, #170d24 0%, #100916 45%, #0d1524 100%);
    padding: 1.1rem;
    box-shadow: 0 14px 42px rgba(0, 0, 0, 0.35);
  }

  .hub-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-end;
    margin-bottom: 1rem;
  }

  .hub-title {
    margin: 0;
    color: #fff4d8;
    font-size: 1.08rem;
    font-family: 'Space Mono', 'Consolas', monospace;
    letter-spacing: 0.03em;
  }

  .hub-subtitle {
    margin: 0.3rem 0 0;
    color: #d4bedf;
    font-size: 0.86rem;
  }

  .hub-view-tabs {
    display: inline-flex;
    background: rgba(18, 11, 30, 0.75);
    border: 1px solid #523271;
    border-radius: 999px;
    padding: 0.2rem;
    gap: 0.25rem;
  }

  .hub-view-tab {
    border: none;
    background: transparent;
    color: #a796c3;
    border-radius: 999px;
    padding: 0.45rem 0.95rem;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    cursor: pointer;
  }

  .hub-view-tab.active {
    color: #1b102b;
    background: linear-gradient(120deg, #ffd56b, #ff9f6b);
    box-shadow: 0 0 18px rgba(255, 180, 87, 0.4);
  }

  .stats-banner {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.8rem;
    margin-bottom: 1rem;
  }

  .stat-card {
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    padding: 0.8rem;
    background: rgba(10, 12, 24, 0.55);
  }

  .stat-card.terrain {
    box-shadow: inset 0 0 0 1px rgba(138, 255, 183, 0.12);
  }

  .stat-card.interstellar {
    box-shadow: inset 0 0 0 1px rgba(111, 199, 255, 0.18);
  }

  .stat-card.waiting {
    box-shadow: inset 0 0 0 1px rgba(255, 211, 132, 0.16);
  }

  .stat-label {
    margin: 0;
    font-size: 0.7rem;
    color: #ceb8e6;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .stat-value {
    margin: 0.4rem 0 0;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 1.5rem;
    color: #f9f3ff;
  }

  .logbook-action-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.95rem;
  }

  .timeline-title {
    margin: 0;
    font-size: 0.95rem;
    color: #f7e9ff;
    font-family: 'Space Mono', 'Consolas', monospace;
  }

  .timeline-subtitle {
    margin: 0.2rem 0 0;
    color: #bfaccf;
    font-size: 0.82rem;
  }

  .timeline-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .timeline-column {
    border: 1px solid #462762;
    border-radius: 10px;
    background: rgba(9, 7, 18, 0.55);
    padding: 0.7rem;
    min-height: 210px;
  }

  .timeline-column-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.6rem;
  }

  .timeline-column-head h4 {
    margin: 0;
    color: #f1e4ff;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .count-chip {
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #e8dbf7;
    font-size: 0.72rem;
    padding: 0.1rem 0.5rem;
  }

  .timeline-empty {
    margin: 0.25rem 0;
    color: #9787ad;
    font-size: 0.82rem;
  }

  .log-card {
    border-radius: 9px;
    border: 1px solid #47305f;
    background: #130f22;
    padding: 0.6rem;
    margin-bottom: 0.55rem;
  }

  .log-card.active {
    box-shadow: inset 0 0 0 1px rgba(112, 185, 255, 0.14);
  }

  .log-card.pending {
    opacity: 0.86;
    box-shadow: inset 0 0 0 1px rgba(255, 220, 120, 0.2);
  }

  .log-card.archived {
    box-shadow: inset 0 0 0 1px rgba(150, 255, 186, 0.18);
  }

  .log-card-top {
    display: flex;
    justify-content: space-between;
    gap: 0.55rem;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .log-location {
    margin: 0;
    color: #f8eeff;
    font-size: 0.84rem;
    font-weight: 700;
  }

  .log-type {
    border-radius: 999px;
    border: 1px solid rgba(142, 225, 181, 0.3);
    color: #b9ffd7;
    font-size: 0.66rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 0.12rem 0.45rem;
  }

  .log-type.interstellar {
    border-color: rgba(113, 206, 255, 0.35);
    color: #abdeff;
  }

  .log-meta {
    margin: 0;
    font-size: 0.72rem;
    color: #ad99c2;
  }

  .log-block-title {
    margin: 0.5rem 0 0.2rem;
    color: #dfc6f9;
    font-size: 0.7rem;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    font-family: 'Space Mono', 'Consolas', monospace;
  }

  .log-block-text {
    margin: 0;
    color: #e7d8f6;
    font-size: 0.8rem;
    line-height: 1.38;
    white-space: pre-wrap;
  }

  .badge-waiting,
  .badge-official {
    border-radius: 999px;
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    padding: 0.16rem 0.46rem;
  }

  .badge-waiting {
    color: #1f1404;
    background: linear-gradient(120deg, #ffe08a, #ffc572);
  }

  .badge-official {
    color: #041f13;
    background: linear-gradient(120deg, #b6ffbe, #77e9bf);
  }

  .directory-panel {
    border: 1px solid #47305f;
    border-radius: 10px;
    padding: 0.8rem;
    background: rgba(9, 8, 17, 0.55);
  }

  .directory-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 0.8rem;
    margin-bottom: 0.8rem;
  }

  .directory-filters {
    display: flex;
    gap: 0.55rem;
    flex-wrap: wrap;
  }

  .directory-search,
  .directory-select {
    border-radius: 7px;
    border: 1px solid #5b3a7b;
    background: #140f22;
    color: #f2e8ff;
    padding: 0.44rem 0.6rem;
    font-size: 0.82rem;
  }

  .directory-search {
    min-width: 220px;
  }

  .manifest-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.72rem;
  }

  .crew-card {
    border: 1px solid #563877;
    border-radius: 10px;
    background:
      radial-gradient(180px 90px at 80% -10%, rgba(255, 207, 122, 0.22), transparent 70%),
      #110d1f;
    color: #efe1ff;
    padding: 0.72rem;
    text-align: left;
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  .crew-card:hover {
    transform: translateY(-2px);
    border-color: #ffcf7a;
  }

  .crew-name {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
  }

  .crew-status,
  .crew-assignment {
    margin: 0.35rem 0 0;
    font-size: 0.78rem;
    color: #d7c5ea;
  }

  .crew-badges {
    margin-top: 0.6rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .crew-badge {
    display: inline-flex;
    border-radius: 999px;
    padding: 0.16rem 0.52rem;
    font-size: 0.7rem;
    border: 1px solid transparent;
  }

  .crew-badge.terrain {
    color: #cbffd6;
    border-color: rgba(145, 255, 174, 0.35);
    background: rgba(39, 108, 70, 0.24);
  }

  .crew-badge.interstellar {
    color: #cce9ff;
    border-color: rgba(122, 185, 255, 0.35);
    background: rgba(48, 93, 151, 0.28);
  }

  .crew-modal {
    display: grid;
    gap: 0.55rem;
  }

  .crew-modal-name {
    margin: 0;
    color: #efe4ff;
    font-weight: 700;
    font-size: 0.98rem;
  }

  .crew-modal-status,
  .crew-modal-line {
    margin: 0;
    color: #ccb9df;
    font-size: 0.85rem;
  }

  .crew-modal-badges {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .crew-modal-visits ul {
    margin: 0.35rem 0 0;
    padding-left: 1.05rem;
    color: #dcccf0;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .btn-primary {
    background: linear-gradient(125deg, #ffd56b, #ff9f6b);
    border: none;
    border-radius: 6px;
    color: #291706;
    cursor: pointer;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.74rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    padding: 0.62rem 1.25rem;
    text-transform: uppercase;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: none;
    border: 1px solid #4f3468;
    border-radius: 6px;
    color: #c4b0d8;
    cursor: pointer;
    font-size: 0.84rem;
    padding: 0.62rem 1.25rem;
  }

  .btn-action {
    background: none;
    border: 1px solid #efb764;
    border-radius: 6px;
    color: #efb764;
    cursor: pointer;
    font-family: 'Space Mono', 'Consolas', monospace;
    font-size: 0.7rem;
    letter-spacing: 0.06em;
    padding: 0.25rem 0.75rem;
  }

  .btn-action:hover {
    background: rgba(239, 183, 100, 0.14);
  }

  .info-block {
    background: rgba(255, 184, 107, 0.07);
    border: 1px solid #4e335f;
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
  }

  .info-text {
    font-size: 0.85rem;
    color: #dac8eb;
  }

  .auth-badge {
    border-radius: 3px;
    font-size: 0.8rem;
    font-weight: 700;
    padding: 0.35rem 0.625rem;
  }

  .auth-ok {
    background: rgba(0, 200, 83, 0.12);
    color: #84ffb8;
  }

  .auth-denied {
    background: rgba(255, 68, 102, 0.12);
    color: #ff9bb0;
  }

  @media (max-width: 1000px) {
    .timeline-grid {
      grid-template-columns: 1fr;
    }

    .stats-banner {
      grid-template-columns: 1fr;
    }

    .hub-header,
    .directory-toolbar,
    .logbook-action-row {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
