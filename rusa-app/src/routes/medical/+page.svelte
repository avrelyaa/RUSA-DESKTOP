<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { medicalApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'shifts' | 'inventory' | 'specializations' | 'budget';
  let activeTab = $state<Tab>('shifts');

  let shifts: any[] = $state([]);
  let inventory: any[] = $state([]);
  let specializations: any[] = $state([]);
  let loading = $state(false);

  // Shift form
  let shiftOpen = $state(false);
  let shiftStaff = $state('');
  let shiftStart = $state('');
  let shiftEnd = $state('');
  let shiftNotes = $state('');

  // Inventory form
  let invOpen = $state(false);
  let invItem = $state('');
  let invCategory = $state('');
  let invQty = $state(0);
  let invUnit = $state('');
  let invExpiry = $state('');

  // Specialization form
  let specOpen = $state(false);
  let specStaff = $state('');
  let specName = $state('');
  let specDate = $state('');

  // Budget form
  let budgetOpen = $state(false);
  let budgetTitle = $state('');
  let budgetDesc = $state('');
  let budgetAmount = $state(0);
  let budgetItems = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [shifts, inventory, specializations] = await Promise.all([
        medicalApi.getShifts(s.token),
        medicalApi.getInventory(s.token),
        medicalApi.getSpecializations(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function allocateShift() {
    const s = $session; if (!s) return;
    if (!shiftStaff || !shiftStart || !shiftEnd) { showToast('Staff ID, start, and end required', 'error'); return; }
    try {
      await medicalApi.allocateShift(s.token, shiftStaff, shiftStart, shiftEnd, shiftNotes || undefined);
      showToast('Shift allocated', 'success');
      shiftOpen = false; shiftStaff = ''; shiftStart = ''; shiftEnd = ''; shiftNotes = '';
      shifts = await medicalApi.getShifts(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function updateInventory() {
    const s = $session; if (!s) return;
    if (!invItem) { showToast('Item name required', 'error'); return; }
    try {
      await medicalApi.updateInventory(s.token, invItem, invCategory || undefined, invQty || undefined, invUnit || undefined, invExpiry || undefined);
      showToast('Inventory updated', 'success');
      invOpen = false; invItem = ''; invCategory = ''; invQty = 0; invUnit = ''; invExpiry = '';
      inventory = await medicalApi.getInventory(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function addSpecialization() {
    const s = $session; if (!s) return;
    if (!specStaff || !specName) { showToast('Staff ID and specialization required', 'error'); return; }
    try {
      await medicalApi.addSpecialization(s.token, specStaff, specName, specDate || undefined);
      showToast('Specialization added', 'success');
      specOpen = false; specStaff = ''; specName = ''; specDate = '';
      specializations = await medicalApi.getSpecializations(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitBudget() {
    const s = $session; if (!s) return;
    if (!budgetTitle || !budgetDesc || !budgetAmount) { showToast('Title, description, and amount required', 'error'); return; }
    try {
      await medicalApi.submitBudget(s.token, budgetTitle, budgetDesc, budgetAmount, budgetItems || undefined);
      showToast('Budget request submitted', 'success');
      budgetOpen = false; budgetTitle = ''; budgetDesc = ''; budgetAmount = 0; budgetItems = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const shiftCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'shift_start', label: 'Start' },
    { key: 'shift_end', label: 'End' },
    { key: 'notes', label: 'Notes' },
  ];
  const invCols = [
    { key: 'item_name', label: 'Item' },
    { key: 'category', label: 'Category' },
    { key: 'quantity', label: 'Qty' },
    { key: 'expiry_date', label: 'Expiry' },
  ];
  const specCols = [
    { key: 'staff_name', label: 'Staff' },
    { key: 'specialization', label: 'Specialization' },
    { key: 'certified_at', label: 'Certified' },
  ];
</script>

<svelte:head><title>RUSA IMS — Medical Services</title></svelte:head>

<PageShell title="Medical Services" subtitle="Staff shifts, inventory, specializations, and budget">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='shifts'} onclick={() => activeTab='shifts'}>Shifts</button>
    <button class="tab" class:active={activeTab==='inventory'} onclick={() => activeTab='inventory'}>Inventory</button>
    <button class="tab" class:active={activeTab==='specializations'} onclick={() => activeTab='specializations'}>Specializations</button>
    <button class="tab" class:active={activeTab==='budget'} onclick={() => activeTab='budget'}>Budget</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'shifts'}
    <div class="section-bar">
      <h2 class="section-title">Staff Shifts</h2>
      <button class="btn-primary" onclick={() => shiftOpen = true}>+ Allocate Shift</button>
    </div>
    <Table columns={shiftCols} rows={shifts} />
  {:else if activeTab === 'inventory'}
    <div class="section-bar">
      <h2 class="section-title">Medical Inventory</h2>
      <button class="btn-primary" onclick={() => invOpen = true}>+ Update Inventory</button>
    </div>
    <Table columns={invCols} rows={inventory} />
  {:else if activeTab === 'specializations'}
    <div class="section-bar">
      <h2 class="section-title">Staff Specializations</h2>
      <button class="btn-primary" onclick={() => specOpen = true}>+ Add Specialization</button>
    </div>
    <Table columns={specCols} rows={specializations} />
  {:else if activeTab === 'budget'}
    <div class="section-bar">
      <h2 class="section-title">Budget Requests</h2>
      <button class="btn-primary" onclick={() => budgetOpen = true}>+ Submit Budget Request</button>
    </div>
    <p class="info-note">Submit budget requests for review by the finance team.</p>
  {/if}
</PageShell>

<Modal bind:open={shiftOpen} title="Allocate Shift">
  <div class="form">
    <Field label="Staff User ID" bind:value={shiftStaff} required />
    <Field label="Shift Start" type="datetime-local" bind:value={shiftStart} required />
    <Field label="Shift End" type="datetime-local" bind:value={shiftEnd} required />
    <Field label="Notes" type="textarea" bind:value={shiftNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => shiftOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={allocateShift}>Allocate</button>
    </div>
  </div>
</Modal>

<Modal bind:open={invOpen} title="Update Medical Inventory">
  <div class="form">
    <Field label="Item Name" bind:value={invItem} required />
    <Field label="Category" bind:value={invCategory} placeholder="e.g. medications, equipment" />
    <Field label="Quantity" type="number" bind:value={invQty} />
    <Field label="Unit" bind:value={invUnit} placeholder="e.g. units, mg, litres" />
    <Field label="Expiry Date" type="date" bind:value={invExpiry} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateInventory}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={specOpen} title="Add Specialization">
  <div class="form">
    <Field label="Staff User ID" bind:value={specStaff} required />
    <Field label="Specialization" bind:value={specName} required placeholder="e.g. Surgery, Neurology" />
    <Field label="Certification Date" type="date" bind:value={specDate} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => specOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={addSpecialization}>Add</button>
    </div>
  </div>
</Modal>

<Modal bind:open={budgetOpen} title="Submit Budget Request">
  <div class="form">
    <Field label="Title" bind:value={budgetTitle} required />
    <Field label="Description" type="textarea" bind:value={budgetDesc} required />
    <Field label="Amount (USD)" type="number" bind:value={budgetAmount} required />
    <Field label="Line Items" type="textarea" bind:value={budgetItems} hint="Itemized breakdown" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => budgetOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitBudget}>Submit</button>
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
  .info-note { color: #4a5d82; font-size: 0.85rem; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
