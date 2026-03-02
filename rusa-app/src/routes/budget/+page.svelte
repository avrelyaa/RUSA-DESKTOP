<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { budgetApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'requests' | 'expenditures';
  let activeTab = $state<Tab>('requests');

  let requests: any[] = $state([]);
  let expenditures: any[] = $state([]);
  let loading = $state(false);

  // Review modals
  let reviewOpen = $state(false);
  let selectedItem: any = $state(null);
  let reviewType = $state<'budget' | 'expenditure'>('budget');
  let reviewStatus = $state('');
  let reviewNotes = $state('');
  let flagReason = $state('');
  let flagOpen = $state(false);

  // Investigation form
  let invOpen = $state(false);
  let invTitle = $state('');
  let invDesc = $state('');
  let invRelated = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [requests, expenditures] = await Promise.all([
        budgetApi.getBudgetRequests(s.token),
        budgetApi.getExpenditures(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  function openReview(item: any, type: 'budget' | 'expenditure') {
    selectedItem = item; reviewType = type; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  function openFlag(item: any, type: 'budget' | 'expenditure') {
    selectedItem = item; reviewType = type; flagReason = ''; flagOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !selectedItem) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      await budgetApi.reviewBudget(s.token, selectedItem.id, reviewStatus, reviewNotes || undefined);
      showToast('Review submitted', 'success'); reviewOpen = false;
      requests = await budgetApi.getBudgetRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitFlag() {
    const s = $session; if (!s || !selectedItem) return;
    if (!flagReason) { showToast('Reason required', 'error'); return; }
    try {
      if (reviewType === 'budget') {
        await budgetApi.flagBudget(s.token, selectedItem.id, flagReason);
        requests = await budgetApi.getBudgetRequests(s.token);
      } else {
        await budgetApi.flagExpenditure(s.token, selectedItem.id, flagReason);
        expenditures = await budgetApi.getExpenditures(s.token);
      }
      showToast('Flagged for review', 'warning'); flagOpen = false;
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitInvestigation() {
    const s = $session; if (!s) return;
    if (!invTitle || !invDesc) { showToast('Title and description required', 'error'); return; }
    try {
      await budgetApi.submitInvestigation(s.token, invTitle, invDesc, invRelated || undefined);
      showToast('Investigation submitted', 'success');
      invOpen = false; invTitle = ''; invDesc = ''; invRelated = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const reqCols = [
    { key: 'title', label: 'Title' },
    { key: 'department', label: 'Department' },
    { key: 'amount', label: 'Amount' },
    { key: 'status', label: 'Status' },
  ];
  const expCols = [
    { key: 'title', label: 'Title' },
    { key: 'department', label: 'Department' },
    { key: 'total_amount', label: 'Amount' },
    { key: 'status', label: 'Status' },
  ];
  const statusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'pending', label: 'Pending' },
  ];
</script>

<svelte:head><title>RUSA IMS — Budget & Finance</title></svelte:head>

<PageShell title="Budget & Finance" subtitle="Budget requests, expenditure reports, and investigations">
  <div class="header-row">
    <div class="tabs">
      <button class="tab" class:active={activeTab==='requests'} onclick={() => activeTab='requests'}>Budget Requests</button>
      <button class="tab" class:active={activeTab==='expenditures'} onclick={() => activeTab='expenditures'}>Expenditure Reports</button>
    </div>
    <button class="btn-warning" onclick={() => invOpen = true}>+ Submit Investigation</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'requests'}
    <Table columns={reqCols} rows={requests} onRowClick={(r) => openReview(r, 'budget')} />
  {:else if activeTab === 'expenditures'}
    <Table columns={expCols} rows={expenditures} onRowClick={(r) => openReview(r, 'expenditure')} />
  {/if}
</PageShell>

<Modal bind:open={reviewOpen} title="Review: {selectedItem?.title}">
  <div class="form">
    <p class="info-text">Amount: <strong>{selectedItem?.amount || selectedItem?.total_amount}</strong></p>
    <Field label="Decision" type="select" bind:value={reviewStatus} options={statusOpts} required />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      <button class="btn-warning" onclick={() => { reviewOpen = false; openFlag(selectedItem, reviewType); }}>Flag</button>
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Review</button>
    </div>
  </div>
</Modal>

<Modal bind:open={flagOpen} title="Flag for Investigation">
  <div class="form">
    <Field label="Reason" type="textarea" bind:value={flagReason} required rows={4} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => flagOpen = false}>Cancel</button>
      <button class="btn-warning" onclick={submitFlag}>Flag</button>
    </div>
  </div>
</Modal>

<Modal bind:open={invOpen} title="Submit Investigation">
  <div class="form">
    <Field label="Title" bind:value={invTitle} required />
    <Field label="Description" type="textarea" bind:value={invDesc} required rows={4} />
    <Field label="Related Report ID (optional)" bind:value={invRelated} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => invOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitInvestigation}>Submit</button>
    </div>
  </div>
</Modal>

<style>
  .header-row { display: flex; align-items: flex-end; justify-content: space-between; margin-bottom: 1.5rem; }
  .tabs { display: flex; gap: 0; border-bottom: 1px solid #1e2d4a; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; color: #8fa3cc; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.08em; padding: 0.75rem 1.25rem; text-transform: uppercase; transition: all 0.15s; }
  .tab.active { color: #00d4ff; border-bottom-color: #00d4ff; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-warning { background: none; border: 1px solid #ffaa00; border-radius: 4px; color: #ffaa00; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
