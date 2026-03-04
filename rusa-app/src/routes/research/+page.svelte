<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { researchApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'experiments' | 'species' | 'tests';
  let activeTab = $state<Tab>('experiments');

  let experiments: any[] = $state([]);
  let species: any[] = $state([]);
  let tests: any[] = $state([]);
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

  onMount(async () => {
    const s = $session;
    if (!s) return;
    loading = true;
    try {
      [experiments, species, tests] = await Promise.all([
        researchApi.getExperiments(s.token),
        researchApi.getSpeciesArchive(s.token),
        researchApi.getTestArchive(s.token)
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
    { value: 'under_review', label: 'Under Review' },
    { value: 'pending', label: 'Pending' },
  ];
</script>

<svelte:head><title>RUSA IMS — Research & Lab</title></svelte:head>

<PageShell title="Research & Lab" subtitle="Experiments, species archive, and test proposals">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='experiments'} onclick={() => activeTab='experiments'}>Experiments</button>
    <button class="tab" class:active={activeTab==='species'} onclick={() => activeTab='species'}>Species Archive</button>
    <button class="tab" class:active={activeTab==='tests'} onclick={() => activeTab='tests'}>Test Archive</button>
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
    {#if reviewTarget?.status === 'pending' && reviewType === 'test'}
      <div class="pending-vote-notice">
        <span class="status-badge status-pending">⏳ Pending Director Vote</span>
        <p class="hint-text">This test proposal is awaiting a Governance vote. Approval or rejection is handled automatically once quorum is reached.</p>
      </div>
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => reviewOpen = false}>Close</button>
      </div>
    {:else}
      <Field label="Status" type="select" bind:value={reviewStatus} options={reviewStatusOpts} required />
      <Field label="Notes" type="textarea" bind:value={reviewNotes} />
      <div class="form-actions">
        <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
        <button class="btn-primary" onclick={submitReview}>Submit Review</button>
      </div>
    {/if}
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
  .pending-vote-notice { display: flex; flex-direction: column; gap: 0.5rem; padding: 0.75rem; background: rgba(0,212,255,0.05); border: 1px solid rgba(0,212,255,0.2); border-radius: 6px; }
  .hint-text { font-size: 0.8rem; color: #6a7f9a; margin: 0; }
  .status-badge { display: inline-block; font-size: 0.7rem; font-family: 'Space Mono', monospace; padding: 2px 8px; border-radius: 10px; text-transform: uppercase; letter-spacing: 0.06em; }
  .status-pending { background: rgba(255,170,0,0.12); color: #ffaa00; }
</style>
