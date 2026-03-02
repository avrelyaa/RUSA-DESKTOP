<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { astronautApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'missions' | 'journals' | 'sectors';
  let activeTab = $state<Tab>('missions');

  let missions: any[] = $state([]);
  let journals: any[] = $state([]);
  let sectors: any[] = $state([]);
  let ships: any[] = $state([]);
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
  let jTitle = $state('');
  let jContent = $state('');
  let jPublic = $state(false);

  // Sector form
  let sectorOpen = $state(false);
  let secName = $state('');
  let secDesc = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [missions, journals, sectors, ships] = await Promise.all([
        astronautApi.getMissions(s.token),
        astronautApi.getJournals(s.token),
        astronautApi.getSectors(s.token),
        astronautApi.getShips(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function createMission() {
    const s = $session; if (!s) return;
    if (!mTitle) { showToast('Title required', 'error'); return; }
    try {
      await astronautApi.createMission(s.token, mTitle, mDesc || undefined, mType || undefined, mShip || undefined, mSector || undefined, undefined, mStart || undefined);
      showToast('Mission created', 'success');
      missionOpen = false; mTitle = ''; mDesc = ''; mType = ''; mShip = ''; mSector = ''; mStart = '';
      missions = await astronautApi.getMissions(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createJournal() {
    const s = $session; if (!s) return;
    if (!jTitle || !jContent) { showToast('Title and content required', 'error'); return; }
    try {
      await astronautApi.createJournal(s.token, undefined, jTitle, jContent, jPublic);
      showToast('Journal created', 'success');
      journalOpen = false; jTitle = ''; jContent = ''; jPublic = false;
      journals = await astronautApi.getJournals(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function createSector() {
    const s = $session; if (!s) return;
    if (!secName) { showToast('Name required', 'error'); return; }
    try {
      await astronautApi.createSector(s.token, secName, secDesc || undefined);
      showToast('Sector created', 'success');
      sectorOpen = false; secName = ''; secDesc = '';
      sectors = await astronautApi.getSectors(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const missionCols = [
    { key: 'title', label: 'Mission' },
    { key: 'mission_type', label: 'Type' },
    { key: 'status', label: 'Status' },
    { key: 'start_date', label: 'Start Date' },
  ];
  const journalCols = [
    { key: 'title', label: 'Title' },
    { key: 'is_public', label: 'Public' },
    { key: 'created_at', label: 'Created' },
  ];
  const sectorCols = [
    { key: 'name', label: 'Sector Name' },
    { key: 'description', label: 'Description' },
  ];
</script>

<svelte:head><title>RUSA IMS — Astronautics</title></svelte:head>

<PageShell title="Astronautics" subtitle="Missions, exploration journals, and sector management">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='missions'} onclick={() => activeTab='missions'}>Missions</button>
    <button class="tab" class:active={activeTab==='journals'} onclick={() => activeTab='journals'}>Journals</button>
    <button class="tab" class:active={activeTab==='sectors'} onclick={() => activeTab='sectors'}>Sectors</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'missions'}
    <div class="section-bar">
      <h2 class="section-title">Missions</h2>
      <button class="btn-primary" onclick={() => missionOpen = true}>+ Create Mission</button>
    </div>
    <Table columns={missionCols} rows={missions} />
  {:else if activeTab === 'journals'}
    <div class="section-bar">
      <h2 class="section-title">Exploration Journals</h2>
      <button class="btn-primary" onclick={() => journalOpen = true}>+ New Journal</button>
    </div>
    <Table columns={journalCols} rows={journals} />
  {:else if activeTab === 'sectors'}
    <div class="section-bar">
      <h2 class="section-title">Sectors</h2>
      <button class="btn-primary" onclick={() => sectorOpen = true}>+ Create Sector</button>
    </div>
    <Table columns={sectorCols} rows={sectors} />
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
      <button class="btn-secondary" onclick={() => missionOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createMission}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={journalOpen} title="New Exploration Journal">
  <div class="form">
    <Field label="Title" bind:value={jTitle} required />
    <Field label="Content" type="textarea" bind:value={jContent} rows={6} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => journalOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createJournal}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={sectorOpen} title="Create Sector">
  <div class="form">
    <Field label="Sector Name" bind:value={secName} required />
    <Field label="Description" type="textarea" bind:value={secDesc} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => sectorOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createSector}>Create</button>
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
