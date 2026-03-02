<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { settlementApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'tasks' | 'supply' | 'anomalies';
  let activeTab = $state<Tab>('tasks');

  let tasks: any[] = $state([]);
  let supplyReqs: any[] = $state([]);
  let anomalies: any[] = $state([]);
  let loading = $state(false);

  // Task form
  let taskOpen = $state(false);
  let tAssignee = $state('');
  let tTitle = $state('');
  let tDesc = $state('');
  let tDue = $state('');

  // Supply request
  let supplyOpen = $state(false);
  let sTitle = $state('');
  let sDesc = $state('');
  let sItems = $state('');

  // Anomaly form
  let anomalyOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aSeverity = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [tasks, supplyReqs, anomalies] = await Promise.all([
        settlementApi.getTasks(s.token),
        settlementApi.getTasks(s.token), // reusing, in real app would be separate
        settlementApi.getTasks(s.token),
      ]);
      [tasks, supplyReqs, anomalies] = await Promise.all([
        settlementApi.getTasks(s.token),
        (async () => { try { return await settlementApi.submitSupplyRequest ? [] : []; } catch { return []; } })(),
        (async () => { try { return []; } catch { return []; } })(),
      ]);
      tasks = await settlementApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!tAssignee || !tTitle) { showToast('Assignee and title required', 'error'); return; }
    try {
      await settlementApi.assignTask(s.token, tAssignee, tTitle, tDesc || undefined, tDue || undefined);
      showToast('Task assigned', 'success');
      taskOpen = false; tAssignee = ''; tTitle = ''; tDesc = ''; tDue = '';
      tasks = await settlementApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitSupply() {
    const s = $session; if (!s) return;
    if (!sTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitSupplyRequest(s.token, undefined, sTitle, sDesc || undefined, sItems || undefined);
      showToast('Supply request submitted', 'success');
      supplyOpen = false; sTitle = ''; sDesc = ''; sItems = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitAnomaly() {
    const s = $session; if (!s) return;
    if (!aTitle) { showToast('Title required', 'error'); return; }
    try {
      await settlementApi.submitAnomalyReport(s.token, undefined, aTitle, aDesc || undefined, aSeverity || undefined);
      showToast('Anomaly reported', 'success');
      anomalyOpen = false; aTitle = ''; aDesc = ''; aSeverity = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const taskCols = [
    { key: 'title', label: 'Task' },
    { key: 'assigned_to_name', label: 'Assigned To' },
    { key: 'status', label: 'Status' },
    { key: 'due_date', label: 'Due' },
  ];
  const severityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
</script>

<svelte:head><title>RUSA IMS — Settlement Operations</title></svelte:head>

<PageShell title="Settlement Operations" subtitle="Task management, supply requests, and anomaly reports">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='tasks'} onclick={() => activeTab='tasks'}>Tasks</button>
    <button class="tab" class:active={activeTab==='supply'} onclick={() => activeTab='supply'}>Supply Requests</button>
    <button class="tab" class:active={activeTab==='anomalies'} onclick={() => activeTab='anomalies'}>Anomaly Reports</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'tasks'}
    <div class="section-bar">
      <h2 class="section-title">Settler Tasks</h2>
      <button class="btn-primary" onclick={() => taskOpen = true}>+ Assign Task</button>
    </div>
    <Table columns={taskCols} rows={tasks} />
  {:else if activeTab === 'supply'}
    <div class="section-bar">
      <h2 class="section-title">Supply Requests</h2>
      <button class="btn-primary" onclick={() => supplyOpen = true}>+ New Request</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'status',label:'Status'},{key:'items',label:'Items'}]} rows={supplyReqs} />
  {:else if activeTab === 'anomalies'}
    <div class="section-bar">
      <h2 class="section-title">Anomaly Reports</h2>
      <button class="btn-primary" onclick={() => anomalyOpen = true}>+ Report Anomaly</button>
    </div>
    <Table columns={[{key:'title',label:'Title'},{key:'severity',label:'Severity'},{key:'status',label:'Status'}]} rows={anomalies} />
  {/if}
</PageShell>

<Modal bind:open={taskOpen} title="Assign Task">
  <div class="form">
    <Field label="Assign To (User ID)" bind:value={tAssignee} required />
    <Field label="Title" bind:value={tTitle} required />
    <Field label="Description" type="textarea" bind:value={tDesc} />
    <Field label="Due Date" type="date" bind:value={tDue} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => taskOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={supplyOpen} title="Submit Supply Request">
  <div class="form">
    <Field label="Title" bind:value={sTitle} required />
    <Field label="Description" type="textarea" bind:value={sDesc} />
    <Field label="Items" type="textarea" bind:value={sItems} hint="List items, one per line" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => supplyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitSupply}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={anomalyOpen} title="Report Anomaly">
  <div class="form">
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <Field label="Severity" type="select" bind:value={aSeverity} options={severityOpts} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => anomalyOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitAnomaly}>Submit</button>
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
