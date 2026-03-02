<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Table from '$lib/components/Table.svelte';
  import Field from '$lib/components/Field.svelte';
  import { session } from '$lib/stores/auth';
  import { psychiatryApi } from '$lib/api';
  import { showToast } from '$lib/stores/toast';
  import { onMount } from 'svelte';

  type Tab = 'patients' | 'appointments';
  let activeTab = $state<Tab>('patients');

  let patients: any[] = $state([]);
  let appointments: any[] = $state([]);
  let loading = $state(false);

  // Register patient
  let registerOpen = $state(false);
  let patientId = $state('');

  // Schedule appointment
  let apptOpen = $state(false);
  let apptPatient = $state('');
  let apptTime = $state('');

  // Complete appointment
  let completeOpen = $state(false);
  let selectedAppt: any = $state(null);
  let findings = $state('');

  onMount(async () => {
    const s = $session; if (!s) return;
    loading = true;
    try {
      [patients, appointments] = await Promise.all([
        psychiatryApi.getPatients(s.token),
        psychiatryApi.getAppointments(s.token),
      ]);
    } catch (e: any) { showToast('Failed to load: ' + e, 'error'); }
    loading = false;
  });

  async function registerPatient() {
    const s = $session; if (!s) return;
    if (!patientId) { showToast('Patient ID required', 'error'); return; }
    try {
      await psychiatryApi.registerPatient(s.token, patientId);
      showToast('Patient registered', 'success');
      registerOpen = false; patientId = '';
      patients = await psychiatryApi.getPatients(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function scheduleAppointment() {
    const s = $session; if (!s) return;
    if (!apptPatient || !apptTime) { showToast('Patient and time required', 'error'); return; }
    try {
      await psychiatryApi.scheduleAppointment(s.token, apptPatient, apptTime);
      showToast('Appointment scheduled', 'success');
      apptOpen = false; apptPatient = ''; apptTime = '';
      appointments = await psychiatryApi.getAppointments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  async function completeAppointment() {
    const s = $session; if (!s || !selectedAppt) return;
    try {
      await psychiatryApi.completeAppointment(s.token, selectedAppt.id, findings || undefined);
      showToast('Appointment completed', 'success');
      completeOpen = false; findings = '';
      appointments = await psychiatryApi.getAppointments(s.token);
    } catch (e: any) { showToast('Failed: ' + e, 'error'); }
  }

  const patientCols = [
    { key: 'patient_name', label: 'Patient' },
    { key: 'registered_at', label: 'Registered' },
    { key: 'status', label: 'Status' },
  ];
  const apptCols = [
    { key: 'patient_name', label: 'Patient' },
    { key: 'scheduled_at', label: 'Scheduled' },
    { key: 'status', label: 'Status' },
  ];
</script>

<svelte:head><title>RUSA IMS — Psychiatry</title></svelte:head>

<PageShell title="Psychiatry" subtitle="Patient management and appointment scheduling">
  <div class="tabs">
    <button class="tab" class:active={activeTab==='patients'} onclick={() => activeTab='patients'}>Patients</button>
    <button class="tab" class:active={activeTab==='appointments'} onclick={() => activeTab='appointments'}>Appointments</button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else if activeTab === 'patients'}
    <div class="section-bar">
      <h2 class="section-title">Patients</h2>
      <button class="btn-primary" onclick={() => registerOpen = true}>+ Register Patient</button>
    </div>
    <Table columns={patientCols} rows={patients} />
  {:else if activeTab === 'appointments'}
    <div class="section-bar">
      <h2 class="section-title">Appointments</h2>
      <button class="btn-primary" onclick={() => apptOpen = true}>+ Schedule Appointment</button>
    </div>
    <Table columns={apptCols} rows={appointments} onRowClick={(r) => { selectedAppt = r; findings = ''; completeOpen = true; }} />
  {/if}
</PageShell>

<Modal bind:open={registerOpen} title="Register Patient">
  <div class="form">
    <Field label="Patient User ID" bind:value={patientId} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => registerOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={registerPatient}>Register</button>
    </div>
  </div>
</Modal>

<Modal bind:open={apptOpen} title="Schedule Appointment">
  <div class="form">
    <Field label="Patient ID" bind:value={apptPatient} required />
    <Field label="Scheduled Time" type="datetime-local" bind:value={apptTime} required />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => apptOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={scheduleAppointment}>Schedule</button>
    </div>
  </div>
</Modal>

<Modal bind:open={completeOpen} title="Complete Appointment">
  <div class="form">
    <p class="info-text">Patient: <strong>{selectedAppt?.patient_name}</strong></p>
    <Field label="Findings / Notes" type="textarea" bind:value={findings} rows={5} />
    <div class="form-actions">
      <button class="btn-secondary" onclick={() => completeOpen = false}>Cancel</button>
      <button class="btn-primary" onclick={completeAppointment}>Mark Complete</button>
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
  .info-text { font-size: 0.85rem; color: #8fa3cc; }
  .btn-primary { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.625rem 1.25rem; }
  .btn-secondary { background: none; border: 1px solid #1e2d4a; border-radius: 4px; color: #8fa3cc; cursor: pointer; font-size: 0.85rem; padding: 0.625rem 1.25rem; }
</style>
