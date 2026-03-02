<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { stationApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'inventory' | 'annotations' | 'personnel' | 'findings';
  let activeTab = $state<Tab>('inventory');

  let stations: any[] = $state([]);
  let inventory: any[] = $state([]);
  let annotations: any[] = $state([]);
  let personnelLog: any[] = $state([]);
  let findings: any[] = $state([]);
  let selectedStation = $state('');
  let loading = $state(false);

  // Inventory form
  let invOpen = $state(false);
  let invCategory = $state('');
  let invItem = $state('');
  let invQty = $state(0);
  let invUnit = $state('');

  // Annotation form
  let annotOpen = $state(false);
  let annotSection = $state('');
  let annotDesc = $state('');

  // Findings form
  let findOpen = $state(false);
  let findTitle = $state('');
  let findDesc = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      stations = await stationApi.getStations(s.token);
      if (stations.length > 0) {
        selectedStation = stations[0].id;
        await loadStationData();
      }
      findings = await stationApi.getFindings(s.token, undefined);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function loadStationData() {
    const s = $session; if (!s || !selectedStation) return;
    try {
      [inventory, annotations, personnelLog] = await Promise.all([
        stationApi.getInventory(s.token, selectedStation),
        stationApi.getAnnotations(s.token, selectedStation),
        stationApi.getPersonnelLog(s.token, selectedStation),
      ]);
    } catch {}
  }

  async function updateInventory() {
    const s = $session; if (!s || !selectedStation) return;
    if (!invCategory || !invItem) { showToast('Category and item required', 'error'); return; }
    try {
      await stationApi.updateInventory(s.token, selectedStation, invCategory, invItem, invQty, invUnit || undefined);
      showToast('Inventory updated', 'success');
      invOpen = false; invCategory = ''; invItem = ''; invQty = 0; invUnit = '';
      inventory = await stationApi.getInventory(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addAnnotation() {
    const s = $session; if (!s || !selectedStation) return;
    if (!annotSection) { showToast('Section name required', 'error'); return; }
    try {
      await stationApi.addAnnotation(s.token, selectedStation, annotSection, annotDesc || undefined);
      showToast('Annotation added', 'success');
      annotOpen = false; annotSection = ''; annotDesc = '';
      annotations = await stationApi.getAnnotations(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFindings() {
    const s = $session; if (!s || !selectedStation) return;
    if (!findTitle) { showToast('Title required', 'error'); return; }
    try {
      await stationApi.submitFindings(s.token, selectedStation, findTitle, findDesc || undefined);
      showToast('Findings submitted', 'success');
      findOpen = false; findTitle = ''; findDesc = '';
      findings = await stationApi.getFindings(s.token, selectedStation);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const invCols = [
    { key: 'category', label: 'Category' },
    { key: 'item_name', label: 'Item' },
    { key: 'quantity', label: 'Qty' },
    { key: 'unit', label: 'Unit' },
  ];
  const annotCols = [
    { key: 'section_name', label: 'Section' },
    { key: 'description', label: 'Description' },
  ];
  const personnelCols = [
    { key: 'event_type', label: 'Event' },
    { key: 'user_name', label: 'Personnel' },
    { key: 'notes', label: 'Notes' },
    { key: 'logged_at', label: 'Time' },
  ];
  const findingsCols = [
    { key: 'title', label: 'Title' },
    { key: 'description', label: 'Description' },
  ];
</script>

<svelte:head><title>RUSA IMS — Space Station</title></svelte:head>

<PageShell title="Space Station" subtitle="Inventory, map annotations, personnel log, and findings">
  {#if stations.length > 0}
    <div class="station-select">
      <label class="field-label" for="station-select">Station</label>
      <select id="station-select" class="field-input" bind:value={selectedStation} onchange={loadStationData}>
        {#each stations as st}
          <option value={st.id}>{st.name}</option>
        {/each}
      </select>
    </div>
  {/if}

  <div class="tabs">
    <button class="tab" class:active={activeTab==='inventory'} onclick={() => activeTab='inventory'}>Inventory</button>
    <button class="tab" class:active={activeTab==='annotations'} onclick={() => activeTab='annotations'}>Map Annotations</button>
    <button class="tab" class:active={activeTab==='personnel'} onclick={() => activeTab='personnel'}>Personnel Log</button>
    <button class="tab" class:active={activeTab==='findings'} onclick={() => activeTab='findings'}>Findings</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'inventory'}
    <div class="section-bar">
      <h2 class="section-title">Station Inventory</h2>
      <button class="btn-primary" onclick={() => invOpen = true}>+ Update Inventory</button>
    </div>
    <Table columns={invCols} rows={inventory} />
  {:else if activeTab === 'annotations'}
    <div class="section-bar">
      <h2 class="section-title">Map Annotations</h2>
      <button class="btn-primary" onclick={() => annotOpen = true}>+ Add Annotation</button>
    </div>
    <Table columns={annotCols} rows={annotations} />
  {:else if activeTab === 'personnel'}
    <div class="section-bar">
      <h2 class="section-title">Personnel Log</h2>
    </div>
    <Table columns={personnelCols} rows={personnelLog} />
  {:else if activeTab === 'findings'}
    <div class="section-bar">
      <h2 class="section-title">Station Findings</h2>
      <button class="btn-primary" onclick={() => findOpen = true}>+ Submit Findings</button>
    </div>
    <Table columns={findingsCols} rows={findings} />
  {/if}
</PageShell>

<Modal bind:open={invOpen} title="Update Inventory">
  <div class="form">
    <Field label="Category" bind:value={invCategory} required />
    <Field label="Item Name" bind:value={invItem} required />
    <Field label="Quantity" type="number" bind:value={invQty} />
    <Field label="Unit" bind:value={invUnit} placeholder="e.g. kg, units, litres" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateInventory}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={annotOpen} title="Add Map Annotation">
  <div class="form">
    <Field label="Section Name" bind:value={annotSection} required />
    <Field label="Description" type="textarea" bind:value={annotDesc} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => annotOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addAnnotation}>Add</button>
    </div>
  </div>
</Modal>

<Modal bind:open={findOpen} title="Submit Findings">
  <div class="form">
    <Field label="Title" bind:value={findTitle} required />
    <Field label="Description" type="textarea" bind:value={findDesc} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => findOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitFindings}>Submit</button>
    </div>
  </div>
</Modal>

<style>
  .station-select { display: flex; align-items: center; gap: 1rem; margin-bottom: 1.5rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; white-space: nowrap; }
  .field-input { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; padding: 0.5rem 0.875rem; }
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
