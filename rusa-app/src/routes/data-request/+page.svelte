<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { dataApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';

  let title = $state('');
  let description = $state('');
  let dataType = $state('');
  let loading = $state(false);

  async function submit() {
    const s = $session; if (!s) return;
    if (!title || !description) { showToast('Title and description required', 'error'); return; }
    loading = true;
    try {
      await dataApi.submitRequest(s.token, title, description, dataType || undefined);
      showToast('Data request submitted successfully', 'success');
      title = ''; description = ''; dataType = '';
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
    loading = false;
  }
</script>

<svelte:head><title>RUSA IMS — Request Data</title></svelte:head>

<PageShell title="Request Data" subtitle="Submit a data access request to the data services team">
  <div class="form-card">
    <Field label="Request Title" bind:value={title} required />
    <Field label="Description" type="textarea" bind:value={description} rows={5} required hint="Describe what data you need and why" />
    <Field label="Data Type" bind:value={dataType} placeholder="e.g. telemetry, environmental, biological" />
    <div class="form-actions">
      <button class="btn-primary" onclick={submit} disabled={loading}>
        {loading ? 'Submitting...' : 'Submit Request'}
      </button>
    </div>
  </div>
</PageShell>

<style>
  .form-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; max-width: 600px; display: flex; flex-direction: column; gap: 1.25rem; }
  .form-actions { display: flex; justify-content: flex-end; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
