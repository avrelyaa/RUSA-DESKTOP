<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { dataApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  let requests: any[] = $state([]);
  let loading = $state(false);
  let createOpen = $state(false);
  let respondOpen = $state(false);
  let selectedReq: any = $state(null);

  let title = $state('');
  let description = $state('');
  let dataType = $state('');

  let reviewStatus = $state('');
  let reviewNotes = $state('');
  let responseData = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try { requests = await dataApi.getRequests(s.token); }
    catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function create() {
    const s = $session; if (!s) return;
    if (!title || !description) { showToast('Title and description required', 'error'); return; }
    try {
      await dataApi.submitRequest(s.token, title, description, dataType || undefined);
      showToast('Request submitted', 'success');
      createOpen = false; title = ''; description = ''; dataType = '';
      requests = await dataApi.getRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openRespond(req: any) { selectedReq = req; responseData = ''; reviewNotes = ''; respondOpen = true; }

  async function submitResponse() {
    const s = $session; if (!s || !selectedReq) return;
    if (!responseData) { showToast('Response data required', 'error'); return; }
    try {
      await dataApi.respond(s.token, selectedReq.id, responseData, reviewNotes || undefined);
      showToast('Response submitted', 'success'); respondOpen = false;
      requests = await dataApi.getRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function reviewRequest(req: any, status: string) {
    const s = $session; if (!s) return;
    try {
      await dataApi.review(s.token, req.id, status);
      showToast('Reviewed', 'success');
      requests = await dataApi.getRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const cols = [
    { key: 'title', label: 'Title' },
    { key: 'data_type', label: 'Type' },
    { key: 'status', label: 'Status' },
    { key: 'requested_at', label: 'Requested' },
  ];
</script>

<svelte:head><title>RUSA IMS — Data Services</title></svelte:head>

<PageShell title="Data Services" subtitle="Manage data requests and responses">
  <div class="section-bar">
    <h2 class="section-title">Data Requests</h2>
    <button class="btn-primary" onclick={() => createOpen = true}>+ New Request</button>
  </div>
  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <Table columns={cols} rows={requests} onRowClick={openRespond} />
  {/if}
</PageShell>

<Modal bind:open={createOpen} title="Submit Data Request">
  <div class="form">
    <Field label="Title" bind:value={title} required />
    <Field label="Description" type="textarea" bind:value={description} required />
    <Field label="Data Type" bind:value={dataType} placeholder="e.g. telemetry, environmental" />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => createOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={create}>Submit</button>
    </div>
  </div>
</Modal>

<Modal bind:open={respondOpen} title="Request: {selectedReq?.title}">
  <div class="form">
    <p class="req-status">Status: <strong>{selectedReq?.status}</strong></p>
    <p class="req-desc">{selectedReq?.description}</p>
    <Field label="Response Data" type="textarea" bind:value={responseData} rows={5} />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      <button class="btn-danger" onclick={() => reviewRequest(selectedReq, 'rejected')}>Reject</button>
      <button class="btn-secondary" onclick={() => respondOpen = false}>Close</button>
      <button class="btn-primary" onclick={submitResponse}>Respond</button>
    </div>
  </div>
</Modal>

<style>
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .req-status { font-size: 0.85rem; color: #8fa3cc; }
  .req-desc { font-size: 0.85rem; color: #e8eeff; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-danger { background: none; border: 1px solid #ff4466; border-radius: 4px; color: #ff4466; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
