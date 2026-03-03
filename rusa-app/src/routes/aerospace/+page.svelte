<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { aerospaceApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'work_orders' | 'technical_reports';
  let activeTab = $state<Tab>('work_orders');

  let workOrders: any[] = $state([]);
  let technicalReports: any[] = $state([]);
  let loading = $state(false);

  // New work order form
  let woOpen = $state(false);
  let woTitle = $state('');
  let woDesc = $state('');
  let woPriority = $state('medium');
  let woSystem = $state('');

  // Update status form
  let statusOpen = $state(false);
  let selectedOrderId = $state('');
  let newStatus = $state('open');
  let statusNotes = $state('');

  // Technical report form
  let reportOpen = $state(false);
  let rTitle = $state('');
  let rContent = $state('');
  let rFindings = $state('');
  let rRecommendations = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [workOrders, technicalReports] = await Promise.all([
        aerospaceApi.getWorkOrders(s.token),
        aerospaceApi.getTechnicalReports(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function createWorkOrder() {
    const s = $session; if (!s) return;
    if (!woTitle) { showToast('Title required', 'error'); return; }
    try {
      await aerospaceApi.createWorkOrder(s.token, woTitle, woDesc || undefined, woPriority || undefined, woSystem || undefined);
      showToast('Work order created', 'success');
      woOpen = false; woTitle = ''; woDesc = ''; woPriority = 'medium'; woSystem = '';
      workOrders = await aerospaceApi.getWorkOrders(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openStatusModal(order: any) {
    selectedOrderId = order.id;
    newStatus = order.status ?? 'open';
    statusNotes = order.notes ?? '';
    statusOpen = true;
  }

  async function updateStatus() {
    const s = $session; if (!s) return;
    try {
      await aerospaceApi.updateWorkOrderStatus(s.token, selectedOrderId, newStatus, statusNotes || undefined);
      showToast('Status updated', 'success');
      statusOpen = false;
      workOrders = await aerospaceApi.getWorkOrders(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function submitReport() {
    const s = $session; if (!s) return;
    if (!rTitle || !rContent) { showToast('Title and content required', 'error'); return; }
    try {
      await aerospaceApi.submitTechnicalReport(s.token, rTitle, rContent, rFindings || undefined, rRecommendations || undefined);
      showToast('Report submitted', 'success');
      reportOpen = false; rTitle = ''; rContent = ''; rFindings = ''; rRecommendations = '';
      technicalReports = await aerospaceApi.getTechnicalReports(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function statusBadgeClass(status: string | null | undefined): string {
    switch (status) {
      case 'open': return 'badge badge-open';
      case 'in_progress': return 'badge badge-progress';
      case 'completed': return 'badge badge-done';
      case 'cancelled': return 'badge badge-cancelled';
      default: return 'badge';
    }
  }

  const priorityOpts = [
    { value: 'low', label: 'Low' },
    { value: 'medium', label: 'Medium' },
    { value: 'high', label: 'High' },
    { value: 'critical', label: 'Critical' },
  ];
  const statusOpts = [
    { value: 'open', label: 'Open' },
    { value: 'in_progress', label: 'In Progress' },
    { value: 'completed', label: 'Completed' },
    { value: 'cancelled', label: 'Cancelled' },
  ];
</script>

<svelte:head><title>RUSA IMS — Aerospace Engineering</title></svelte:head>

<PageShell title="Aerospace Engineering" subtitle="Work orders and technical reports">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='work_orders'} onclick={() => activeTab='work_orders'}>Work Orders</button>
    <button class="tab" class:active={activeTab==='technical_reports'} onclick={() => activeTab='technical_reports'}>Technical Reports</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'work_orders'}
    <div class="section-bar">
      <h2 class="section-title">Work Orders</h2>
      {#if $session?.role_name === 'aerospace_engineer'}
        <button class="btn-primary" onclick={() => woOpen = true}>+ New Work Order</button>
      {/if}
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead>
          <tr>
            <th>Title</th>
            <th>Priority</th>
            <th>System</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each workOrders as order}
            <tr>
              <td>{order.title}</td>
              <td>{order.priority ?? '—'}</td>
              <td>{order.system_affected ?? '—'}</td>
              <td><span class={statusBadgeClass(order.status)}>{order.status ?? '—'}</span></td>
              <td>
                {#if $session?.role_name === 'aerospace_engineer'}
                  <button class="btn-small" onclick={() => openStatusModal(order)}>Update Status</button>
                {/if}
              </td>
            </tr>
          {/each}
          {#if workOrders.length === 0}
            <tr><td colspan="5" class="empty">No work orders found.</td></tr>
          {/if}
        </tbody>
      </table>
    </div>
  {:else if activeTab === 'technical_reports'}
    <div class="section-bar">
      <h2 class="section-title">Technical Reports</h2>
      {#if $session?.role_name === 'aerospace_engineer'}
        <button class="btn-primary" onclick={() => reportOpen = true}>+ Submit Report</button>
      {/if}
    </div>
    <div class="reports-list">
      {#each technicalReports as report}
        <div class="report-card">
          <div class="report-header">
            <span class="report-title">{report.title}</span>
            <span class="report-date">{report.created_at ? new Date(report.created_at).toLocaleDateString() : '—'}</span>
          </div>
          {#if report.content}
            <p class="report-body">{report.content}</p>
          {/if}
          {#if report.findings}
            <div class="report-section"><span class="report-label">Findings:</span> {report.findings}</div>
          {/if}
          {#if report.recommendations}
            <div class="report-section"><span class="report-label">Recommendations:</span> {report.recommendations}</div>
          {/if}
        </div>
      {/each}
      {#if technicalReports.length === 0}
        <p class="empty">No technical reports found.</p>
      {/if}
    </div>
  {/if}
</PageShell>

<Modal bind:open={woOpen} title="New Work Order">
  <div class="form">
    <Field label="Title" bind:value={woTitle} required />
    <Field label="Description" type="textarea" bind:value={woDesc} />
    <Field label="Priority" type="select" bind:value={woPriority} options={priorityOpts} />
    <Field label="System Affected" bind:value={woSystem} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => woOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={createWorkOrder}>Create</button>
    </div>
  </div>
</Modal>

<Modal bind:open={statusOpen} title="Update Work Order Status">
  <div class="form">
    <Field label="Status" type="select" bind:value={newStatus} options={statusOpts} />
    <Field label="Notes" type="textarea" bind:value={statusNotes} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => statusOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={updateStatus}>Update</button>
    </div>
  </div>
</Modal>

<Modal bind:open={reportOpen} title="Submit Technical Report">
  <div class="form">
    <Field label="Title" bind:value={rTitle} required />
    <Field label="Content" type="textarea" bind:value={rContent} required />
    <Field label="Findings" type="textarea" bind:value={rFindings} />
    <Field label="Recommendations" type="textarea" bind:value={rRecommendations} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => reportOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitReport}>Submit</button>
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
  .badge-cancelled { background: rgba(255,68,102,0.12); color: #ff4466; }
  .reports-list { display: flex; flex-direction: column; gap: 1rem; }
  .report-card { background: #0a1020; border: 1px solid #1e2d4a; border-radius: 6px; padding: 1rem 1.25rem; }
  .report-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; }
  .report-title { color: #e8eeff; font-family: 'Outfit', sans-serif; font-size: 0.9rem; font-weight: 600; }
  .report-date { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; }
  .report-body { color: #8fa3cc; font-size: 0.85rem; line-height: 1.5; margin-bottom: 0.5rem; }
  .report-section { color: #8fa3cc; font-size: 0.82rem; line-height: 1.4; margin-top: 0.4rem; }
  .report-label { color: #4a5d82; font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.06em; }
</style>
