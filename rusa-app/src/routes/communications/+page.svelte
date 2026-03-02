<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { securityApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  // The Anchorman manages broadcasts and communications
  let broadcasts: any[] = $state([]);
  let loading = $state(false);

  // Review modal
  let reviewOpen = $state(false);
  let selectedBroadcast: any = $state(null);
  let reviewStatus = $state('');
  let reviewNotes = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      broadcasts = await securityApi.getBroadcastRequests(s.token);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  function openReview(item: any) {
    selectedBroadcast = item; reviewStatus = ''; reviewNotes = ''; reviewOpen = true;
  }

  async function submitReview() {
    const s = $session; if (!s || !selectedBroadcast) return;
    if (!reviewStatus) { showToast('Status required', 'error'); return; }
    try {
      await securityApi.reviewBroadcast(s.token, selectedBroadcast.id, reviewStatus, reviewNotes || undefined);
      showToast('Broadcast reviewed', 'success'); reviewOpen = false;
      broadcasts = await securityApi.getBroadcastRequests(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const broadcastCols = [
    { key: 'title', label: 'Title' },
    { key: 'requester_name', label: 'Requested By' },
    { key: 'target_audience', label: 'Audience' },
    { key: 'status', label: 'Status' },
    { key: 'created_at', label: 'Submitted' },
  ];
  const statusOpts = [
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
    { value: 'pending', label: 'Pending' },
  ];
</script>

<svelte:head><title>RUSA IMS — Communications</title></svelte:head>

<PageShell title="Communications" subtitle="Broadcast request management — The Anchorman">
  <div class="section-bar">
    <h2 class="section-title">Broadcast Requests</h2>
    <div class="header-info">Click a request to review it</div>
  </div>
  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <Table columns={broadcastCols} rows={broadcasts} onRowClick={openReview} />
  {/if}
</PageShell>

<Modal bind:open={reviewOpen} title="Review Broadcast: {selectedBroadcast?.title}">
  <div class="form">
    {#if selectedBroadcast}
      <div class="preview-box">
        <p class="preview-label">Content</p>
        <p class="preview-text">{selectedBroadcast.content}</p>
      </div>
      <p class="meta-text">Audience: <strong>{selectedBroadcast.target_audience || 'All'}</strong></p>
    {/if}
    <Field label="Decision" type="select" bind:value={reviewStatus} options={statusOpts} required />
    <Field label="Notes" type="textarea" bind:value={reviewNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reviewOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReview}>Submit Decision</button>
    </div>
  </div>
</Modal>

<style>
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .header-info { font-size: 0.8rem; color: #4a5d82; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .preview-box { background: #05070f; border: 1px solid #1e2d4a; border-radius: 4px; padding: 1rem; }
  .preview-label { font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; margin-bottom: 0.5rem; }
  .preview-text { color: #e8eeff; font-size: 0.9rem; line-height: 1.6; }
  .meta-text { font-size: 0.82rem; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
