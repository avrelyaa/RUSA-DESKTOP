<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { researchTaskApi, userApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  let tasks: any[] = $state([]);
  let loading = $state(false);

  // Assign task form (observer/artificer)
  let assignOpen = $state(false);
  let aTitle = $state('');
  let aDesc = $state('');
  let aAssignTo = $state('');
  let allUsers: any[] = $state([]);
  let assigneeSearch = $state('');
  let assigneeSelected: any = $state(null);
  let aSourceMsg = $state('');
  let aDueDate = $state('');

  // Submit result form (subordinate)
  let resultOpen = $state(false);
  let resultTaskId = $state('');
  let resultNotes = $state('');

  const isDirector = $derived(
    $session?.role_name === 'the_observer' || $session?.role_name === 'the_artificer' || $session?.role_name === 'the_taskmaster'
  );

  function filteredAssignees(search: string): any[] {
    const q = search.toLowerCase();
    return allUsers.filter(u =>
      u.username?.toLowerCase().includes(q) || u.full_name?.toLowerCase().includes(q)
    ).slice(0, 8);
  }

  let assigneeSuggestions = $derived(assigneeSearch.length > 0 ? filteredAssignees(assigneeSearch) : []);

  const taskCols = [
    { key: 'title', label: 'Title' },
    { key: 'status', label: 'Status' },
    { key: 'assigned_to', label: 'Assigned To' },
    { key: 'due_date', label: 'Due Date' },
    { key: 'source_message_id', label: 'From Message' },
  ];

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      tasks = await researchTaskApi.getTasks(s.token);
      allUsers = await userApi.getAll(s.token);
    } catch (e: any) { showToast('Failed to load tasks: ' + e, 'error'); }
    loading = false;
  });

  async function assignTask() {
    const s = $session; if (!s) return;
    if (!aTitle) { showToast('Title required', 'error'); return; }
    try {
      await researchTaskApi.assign(s.token, aTitle, aDesc || undefined, assigneeSelected?.id || undefined, aSourceMsg || undefined, aDueDate || undefined);
      showToast('Task assigned', 'success');
      assignOpen = false; aTitle = ''; aDesc = ''; assigneeSelected = null; assigneeSearch = ''; aSourceMsg = ''; aDueDate = '';
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function openSubmitResult(taskId: string) {
    resultTaskId = taskId;
    resultNotes = '';
    resultOpen = true;
  }

  async function submitResult() {
    const s = $session; if (!s) return;
    if (!resultNotes) { showToast('Result notes required', 'error'); return; }
    try {
      await researchTaskApi.submitResult(s.token, resultTaskId, resultNotes);
      showToast('Result submitted', 'success');
      resultOpen = false; resultTaskId = ''; resultNotes = '';
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function markComplete(taskId: string) {
    const s = $session; if (!s) return;
    try {
      await researchTaskApi.complete(s.token, taskId);
      showToast('Task marked complete', 'success');
      tasks = await researchTaskApi.getTasks(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  function displaySourceId(val: string | null | undefined): string {
    return val ? val : '–';
  }

  const displayTasks = $derived(tasks.map(t => ({
    ...t,
    source_message_id: displaySourceId(t.source_message_id),
  })));
</script>

<svelte:head><title>RUSA IMS — Theoretical Sciences</title></svelte:head>

<PageShell title="Theoretical Sciences" subtitle="Research task assignment and result tracking">
  <div class="section-bar">
    <h2 class="section-title">Research Tasks</h2>
    {#if isDirector}
      <button class="btn-primary" onclick={() => assignOpen = true}>+ Assign Task</button>
    {/if}
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <Table columns={taskCols} rows={displayTasks}>
      {#snippet rowActions(row)}
        {#if isDirector && row.status === 'result_submitted'}
          <button class="btn-action" onclick={() => markComplete(row.id)}>Mark Complete</button>
        {:else if !isDirector && (row.status === 'pending' || row.status === 'in_progress') && row.assigned_to === $session?.user_id}
          <button class="btn-action" onclick={() => openSubmitResult(row.id)}>Submit Result</button>
        {/if}
      {/snippet}
    </Table>
  {/if}
</PageShell>

<Modal bind:open={assignOpen} title="Assign Research Task">
  <div class="form">
    <Field label="Title" bind:value={aTitle} required />
    <Field label="Description" type="textarea" bind:value={aDesc} />
    <div class="field">
      <label class="field-label">Assign To (optional)</label>
      <div class="assignee-autocomplete">
        {#if assigneeSelected}
          <span class="token">{assigneeSelected.full_name || assigneeSelected.username}<button class="token-remove" onclick={() => { assigneeSelected = null; assigneeSearch = ''; }}>×</button></span>
        {:else}
          <div class="autocomplete-wrap">
            <input class="field-input" bind:value={assigneeSearch} placeholder="Search by name or username…" />
            {#if assigneeSuggestions.length > 0}
              <div class="suggestions">
                {#each assigneeSuggestions as u}
                  <button class="suggestion-item" onclick={() => { assigneeSelected = u; assigneeSearch = ''; }}>{u.full_name || u.username} <span class="suggestion-role">@{u.username}</span></button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
    <Field label="Source Message ID" bind:value={aSourceMsg} placeholder="Paste the message ID from the help request" />
    <Field label="Due Date" type="date" bind:value={aDueDate} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => { assignOpen = false; assigneeSelected = null; assigneeSearch = ''; }}>Cancel</button>
      <button class="btn-primary" onclick={assignTask}>Assign</button>
    </div>
  </div>
</Modal>

<Modal bind:open={resultOpen} title="Submit Task Result">
  <div class="form">
    <Field label="Result Notes" type="textarea" bind:value={resultNotes} rows={5} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => resultOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={submitResult}>Submit</button>
    </div>
  </div>
</Modal>

<style>
  .section-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .loading { color: #4a5d82; padding: 2rem 0; }
  .form { display: flex; flex-direction: column; gap: 1rem; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
  .btn-action { background: none; border: 1px solid #3d7fff; border-radius: 4px; color: #3d7fff; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.7rem; letter-spacing: 0.06em; padding: 0.25rem 0.75rem; }
  .btn-action:hover { background: #3d7fff22; }
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label { font-family: 'Space Mono', monospace; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; }
  .assignee-autocomplete { position: relative; }
  .autocomplete-wrap { position: relative; }
  .suggestions { position: absolute; top: 100%; left: 0; right: 0; background: #0d1528; border: 1px solid #1e2d4a; border-radius: 4px; z-index: 100; max-height: 200px; overflow-y: auto; }
  .suggestion-item { display: block; width: 100%; background: none; border: none; border-bottom: 1px solid #1e2d4a; color: #e8eeff; cursor: pointer; font-size: 0.85rem; padding: 0.5rem 0.75rem; text-align: left; }
  .suggestion-item:last-child { border-bottom: none; }
  .suggestion-item:hover { background: rgba(61,127,255,0.1); }
  .suggestion-role { color: #4a5d82; font-size: 0.75rem; }
  .token { display: inline-flex; align-items: center; gap: 0.25rem; background: rgba(61,127,255,0.15); border: 1px solid rgba(61,127,255,0.3); border-radius: 12px; color: #8fa3cc; font-size: 0.8rem; padding: 2px 8px; }
  .token-remove { background: none; border: none; color: #8fa3cc; cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 0 2px; }
  .token-remove:hover { color: #ff4466; }
</style>

