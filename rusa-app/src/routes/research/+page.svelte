<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { researchApi, researchTaskApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'experiments' | 'species' | 'tests' | 'research_tasks';
  let activeTab = $state<Tab>('experiments');

  let experiments: any[] = $state([]);
  let species: any[] = $state([]);
  let tests: any[] = $state([]);
  let researchTasks: any[] = $state([]);
  let loading = $state(false);

  // Experiment form
  let expOpen = $state(false);
  let expTitle = $state('');
  let expDesc = $state('');
  let expType = $state('');
  let expStart = $state('');
  let expEnd = $state('');

  // Species form
  let speciesOpen = $state(false);
  let speciesName = $state('');
  let speciesClass = $state('');
  let speciesDesc = $state('');
  let speciesHabitat = $state('');

  // Test form
  let testOpen = $state(false);
  let testTitle = $state('');
  let testDesc = $state('');
  let testMethod = $state('');

  // Review form
  let reviewOpen = $state(false);
  let reviewTarget: any = $state(null);
  let reviewType = $state<'experiment' | 'test'>('experiment');
  let reviewStatus = $state('');
  let reviewNotes = $state('');

  // Assign research task form
  let assignTaskOpen = $state(false);
  let rtAssignee = $state('');
  let rtTitle = $state('');
  let rtDesc = $state('');
  let rtDue = $state('');

  // Submit result form
  let resultOpen = $state(false);
  let selectedTask: any = $state(null);
  let resultNotes = $state('');

  const canAssignResearchTasks = $derived(
    $session?.role_name === 'the_observer' ||
    $session?.role_name === 'the_artificer' ||
    $session?.role_name === 'the_taskmaster'
  );

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      [experiments, species, tests, researchTasks] = await Promise.all([
        researchApi.getExperiments(s.token),
        researchApi.getSpeciesArchive(s.token),
        researchApi.getTestArchive(s.token),
        researchTaskApi.getTasks(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function proposeExperiment() {
    const s = $session; if (!s) return;
    if (!expTitle || !expDesc || !expType) { showToast('Title, description, and type required', 'error'); return; }
    try {
      await researchApi.proposeExperiment(s.token, expTitle, expDesc, expType, expStart || undefined, expEnd || undefined);
      showToast('Experiment proposed', 'success');
      expOpen = false; expTitle = ''; expDesc = ''; expType = ''; expStart = ''; expEnd = '';
      experiments = await researchApi.getExperiments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addSpecies() {
    const s = $session; if (!s) return;
    if (!speciesName) { showToast('Name required', 'error'); return; }
    try {
      await researchApi.addSpecies(s.token, speciesName, speciesClass || undefined, speciesDesc || undefined, speciesHabitat || undefined);
      showToast('Species added', 'success');
      speciesOpen = false; speciesName = ''; speciesClass = ''; speciesDesc = ''; speciesHabitat = '';
      species = await researchApi.getSpeciesArchive(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function proposeTest() {
    const s = $session; if (!s) return;
    if (!testTitle) { showToast('Title required', 'error'); return; }
    try {
      await researchApi.proposeTest(s.token, testTitle, testDesc || undefined, testMethod || undefined);
      showToast('Test proposed', 'success');
      testOpen = false; testTitle = ''; testDesc = ''; testMethod = '';
      tests = await researchApi.getTestArchive(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openReview(item: any, type: 'experiment' | 'test') {
    reviewTarget = item; reviewType = type; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !reviewTarget) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      if (reviewType === 'experiment') {
        await researchApi.reviewExperiment(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        experiments = await researchApi.getExperiments(s.token);
      } else {
        await researchApi.reviewTest(s.token, reviewTarget.id, reviewStatus, reviewNotes || undefined);
        tests = await researchApi.getTestArchive(s.token);
      }
      showToast('Review submitted', 'success'); reviewOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function assignResearchTask() {
    const s = $session; if (!s) return;
    if (!rtAssignee || !rtTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await researchTaskApi.assign(s.token, rtTitle, rtDesc || undefined, rtAssignee, undefined, rtDue || undefined);
      showToast('Research task assigned', 'success');
      assignTaskOpen = false; rtAssignee = ''; rtTitle = ''; rtDesc = ''; rtDue = '';
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openResultModal(task: any) {
    selectedTask = task;
    resultNotes = task.result_notes ?? '';
    resultOpen = true;
  }

  async function submitResult() {
    const s = $session; if (!s || !selectedTask) return;
    if (!resultNotes) { showToast('Result notes required', 'error'); return; }
    try {
      await researchTaskApi.submitResult(s.token, selectedTask.id, resultNotes);
      showToast('Result submitted', 'success');
      resultOpen = false;
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function completeTask(task: any) {
    const s = $session; if (!s) return;
    try {
      await researchTaskApi.complete(s.token, task.id);
      showToast('Task marked complete', 'success');
      researchTasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'pending': return 'badge badge-open';
      case 'result_submitted': return 'badge badge-progress';
      case 'completed': return 'badge badge-done';
      default: return 'badge';
    }
  }

  const expCols = [
    { key: 'title', label: 'Title' },
    { key: 'experiment_type', label: 'Type' },
    { key: 'status', label: 'Status' },
    { key: 'start_date', label: 'Start Date' },
  ];
  const speciesCols = [
    { key: 'name', label: 'Name' },
    { key: 'classification', label: 'Classification' },
    { key: 'habitat', label: 'Habitat' },
  ];
  const testCols = [
    { key: 'title', label: 'Title' },
    { key: 'status', label: 'Status' },
    { key: 'methodology', label: 'Methodology' },
  ];
  const reviewStatusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'under_review', label: 'Under Review' },
  ];
</script>

<svelte:head><title>RUSA IMS — Research & Lab</title></svelte:head>

<PageShell title="Research & Lab" subtitle="Experiments, species archive, test proposals, and research tasks">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='experiments'} onclick={() => activeTab='experiments'}>Experiments</button>
    <button class="tab" class:active={activeTab==='species'} onclick={() => activeTab='species'}>Species Archive</button>
    <button class="tab" class:active={activeTab==='tests'} onclick={() => activeTab='tests'}>Test Archive</button>
    <button class="tab" class:active={activeTab==='research_tasks'} onclick={() => activeTab='research_tasks'}>
      {canAssignResearchTasks ? 'Research Tasks' : 'My Research Tasks'}
    </button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'experiments'}
    <div class="section-bar">
      <h2 class="section-title">Experiments</h2>
      <button class="btn-primary" onclick={() => expOpen = true}>+ Propose Experiment</button>
    </div>
    <Table columns={expCols} rows={experiments} onRowClick={(r) => openReview(r, 'experiment')} />
  {:else if activeTab === 'species'}
    <div class="section-bar">
      <h2 class="section-title">Species Archive</h2>
      <button class="btn-primary" onclick={() => speciesOpen = true}>+ Add Species</button>
    </div>
    <Table columns={speciesCols} rows={species} />
  {:else if activeTab === 'tests'}
    <div class="section-bar">
      <h2 class="section-title">Test Archive</h2>
      <button class="btn-primary" onclick={() => testOpen = true}>+ Propose Test</button>
    </div>
    <Table columns={testCols} rows={tests} onRowClick={(r) => openReview(r, 'test')} />
  {:else if activeTab === 'research_tasks'}
    <div class="section-bar">
      <h2 class="section-title">{canAssignResearchTasks ? 'Tasks I Assigned' : 'My Research Tasks'}</h2>
      {#if canAssignResearchTasks}
        <button class="btn-primary" onclick={() => assignTaskOpen = true}>+ Assign Task</button>
      {/if}
    </div>
    {#if researchTasks.length === 0}
      <p class="empty">{canAssignResearchTasks ? 'No research tasks assigned yet.' : 'No research tasks have been assigned to you.'}</p>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Status</th>
              <th>Due Date</th>
              <th>Result Notes</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each researchTasks as task}
              <tr>
                <td>{task.title}</td>
                <td><span class={statusBadgeClass(task.status)}>{task.status ?? '—'}</span></td>
                <td>{task.due_date ? new Date(task.due_date).toLocaleDateString() : '—'}</td>
                <td>{task.result_notes ?? '—'}</td>
                <td>
                  {#if !canAssignResearchTasks && task.status === 'pending'}
                    <button class="btn-small" onclick={() => openResultModal(task)}>Submit Result</button>
                  {/if}
                  {#if canAssignResearchTasks && task.status === 'result_submitted'}
                    <button class="btn-small" onclick={() => completeTask(task)}>Mark Complete</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</PageShell>

<Modal bind:open={expOpen} title="Propose Experiment">
  <div class="form">
    <Field label="Title" bind:value={expTitle} required />
    <Field label="Description" type="textarea" bind:value={expDesc} required />
    <Field label="Experiment Type" bind:value={expType} placeholder="e.g. biology, chemistry" required />
    <Field label="Start Date" type="date" bind:value={expStart} />
    <Field label="End Date" type="date" bind:value={expEnd} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => expOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeExperiment}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={speciesOpen} title="Add Species">
  <div class="form">
    <Field label="Name" bind:value={speciesName} required />
    <Field label="Classification" bind:value={speciesClass} />
    <Field label="Description" type="textarea" bind:value={speciesDesc} />
    <Field label="Habitat" bind:value={speciesHabitat} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => speciesOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addSpecies}>Add Species</button>
    </div>
  </div>
</Modal>

<Modal bind:open={testOpen} title="Propose Test">
  <div class="form">
    <Field label="Title" bind:value={testTitle} required />
    <Field label="Description" type="textarea" bind:value={testDesc} />
    <Field label="Methodology" type="textarea" bind:value={testMethod} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => testOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={proposeTest}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={reviewOpen} title="Review {reviewTarget?.title}">
  <div class="form">
    <Field label="Status" type="select" bind:value={reviewStatus} options={reviewStatusOpts} required />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={assignTaskOpen} title="Assign Research Task">
  <div class="form">
    <Field label="Assign To (User ID)" bind:value={rtAssignee} required />
    <Field label="Title" bind:value={rtTitle} required />
    <Field label="Description" type="textarea" bind:value={rtDesc} />
    <Field label="Due Date" type="datetime-local" bind:value={rtDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => assignTaskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={assignResearchTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={resultOpen} title="Submit Task Result">
  <div class="form">
    <p class="info-text">Task: <strong>{selectedTask?.title}</strong></p>
    <Field label="Result Notes" type="textarea" bind:value={resultNotes} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => resultOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitResult}>Submit</button>
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
</style>
