<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import { session } from '$lib/stores/auth';
  import {
    aerospaceApi,
    governanceApi,
    medicalApi,
    messageApi,
    psychiatryApi,
    researchTaskApi,
    sanitaryApi,
    securityApi,
    settlementApi
  } from '$lib/api';
  import { onMount } from 'svelte';

  type PersonalScheduleItem = {
    id: string;
    kind: 'shift' | 'task' | 'appointment';
    source: string;
    title: string;
    starts_at: string | null;
    ends_at: string | null;
    due_at: string | null;
    status: string | null;
    notes: string | null;
    sort_ms: number;
  };

  const MAX_DISPLAYED_SCHEDULE_ITEMS = 12;

  let inbox: any[] = $state([]);
  let broadcasts: any[] = $state([]);
  let scheduleItems: PersonalScheduleItem[] = $state([]);
  let loading = $state(true);
  let scheduleLoading = $state(true);

  function toTimeMs(dateValue: string | null | undefined): number | null {
    if (!dateValue) return null;
    // Treat date-only due dates as end-of-day so they remain visible throughout that day.
    if (/^\d{4}-\d{2}-\d{2}$/.test(dateValue)) {
      const ms = new Date(`${dateValue}T23:59:59`).getTime();
      return Number.isFinite(ms) ? ms : null;
    }
    const ms = new Date(dateValue).getTime();
    return Number.isFinite(ms) ? ms : null;
  }

  function formatDateTime(dateStr: string | null): string {
    return dateStr ? new Date(dateStr).toLocaleString('en-US', { dateStyle: 'short', timeStyle: 'short' }) : '—';
  }

  function asStringId(value: unknown): string {
    return typeof value === 'string' ? value : '';
  }

  function matchesAssignedUser(row: any, userId: string): boolean {
    const assignedTo = asStringId(row?.assigned_to);
    const staffId = asStringId(row?.staff_id);
    const patientId = asStringId(row?.patient_id);
    const psychiatristId = asStringId(row?.psychiatrist_id);
    return assignedTo === userId || staffId === userId || patientId === userId || psychiatristId === userId;
  }

  onMount(async () => {
    const s = $session;
    if (!s) return;
    try {
      const [messages, scheduleSources] = await Promise.all([
        messageApi.getInbox(s.token),
        Promise.allSettled([
          medicalApi.getShifts(s.token),
          sanitaryApi.getShifts(s.token),
          medicalApi.getTasks(s.token),
          sanitaryApi.getTasks(s.token),
          psychiatryApi.getTasks(s.token),
          psychiatryApi.getAppointments(s.token),
          securityApi.getSecurityTasks(s.token),
          settlementApi.getTasks(s.token),
          governanceApi.getNomadTasks(s.token),
          researchTaskApi.getTasks(s.token),
          aerospaceApi.getAssignedTasks(s.token)
        ])
      ]);
      inbox = messages.filter((m: any) => !m.is_broadcast);
      broadcasts = messages.filter((m: any) => m.is_broadcast);

      const [medicalShifts, sanitaryShifts, medicalTasks, sanitaryTasks, psychiatryTasks, appointments, securityTasks, settlerTasks, nomadTasks, researchTasks, aerospaceTasks] =
        scheduleSources.map((result) => (result.status === 'fulfilled' ? result.value : []));

      const collected: PersonalScheduleItem[] = [];

      for (const row of medicalShifts.filter((r: any) => matchesAssignedUser(r, s.user_id))) {
        const startMs = toTimeMs(row.shift_start ?? null);
        if (startMs === null) continue;
        collected.push({
          id: String(row.id),
          kind: 'shift',
          source: 'Medical',
          title: row.staff_name ? `${row.staff_name} Shift` : 'Assigned Shift',
          starts_at: row.shift_start ?? null,
          ends_at: row.shift_end ?? null,
          due_at: null,
          status: null,
          notes: row.notes ?? null,
          sort_ms: startMs
        });
      }

      for (const row of sanitaryShifts.filter((r: any) => matchesAssignedUser(r, s.user_id))) {
        const startMs = toTimeMs(row.shift_start ?? null);
        if (startMs === null) continue;
        collected.push({
          id: String(row.id),
          kind: 'shift',
          source: 'Sanitary',
          title: row.staff_name ? `${row.staff_name} Shift` : 'Assigned Shift',
          starts_at: row.shift_start ?? null,
          ends_at: row.shift_end ?? null,
          due_at: null,
          status: null,
          notes: row.notes ?? null,
          sort_ms: startMs
        });
      }

      for (const row of appointments.filter((r: any) => matchesAssignedUser(r, s.user_id))) {
        const schedMs = toTimeMs(row.scheduled_at ?? null);
        if (schedMs === null) continue;
        collected.push({
          id: String(row.id),
          kind: 'appointment',
          source: 'Psychiatry',
          title: row.patient_name ? `Appointment: ${row.patient_name}` : 'Scheduled Appointment',
          starts_at: row.scheduled_at ?? null,
          ends_at: null,
          due_at: null,
          status: row.status ?? null,
          notes: row.findings ?? null,
          sort_ms: schedMs
        });
      }

      const taskSources: Array<{ source: string; rows: any[] }> = [
        { source: 'Medical', rows: medicalTasks },
        { source: 'Sanitary', rows: sanitaryTasks },
        { source: 'Psychiatry', rows: psychiatryTasks },
        { source: 'Security', rows: securityTasks },
        { source: 'Settlement', rows: settlerTasks },
        { source: 'Governance', rows: nomadTasks },
        { source: 'Research', rows: researchTasks },
        { source: 'Aerospace', rows: aerospaceTasks }
      ];

      for (const source of taskSources) {
        for (const row of source.rows.filter((r: any) => matchesAssignedUser(r, s.user_id))) {
          const dueAt = row.due_date ?? null;
          const dueMs = toTimeMs(dueAt);
          if (dueMs === null) continue;
          collected.push({
            id: String(row.id),
            kind: 'task',
            source: source.source,
            title: row.title ?? 'Assigned Task',
            starts_at: null,
            ends_at: null,
            due_at: dueAt,
            status: row.status ?? null,
            notes: row.description ?? row.progress_notes ?? row.result_notes ?? null,
            sort_ms: dueMs
          });
        }
      }

      const now = Date.now();
      scheduleItems = collected
        .filter((item) => item.sort_ms >= now)
        .sort((a, b) => a.sort_ms - b.sort_ms);
    } catch (e: any) {
      console.error('Failed to load dashboard messages/schedule: ' + e);
    } finally {
      scheduleLoading = false;
    }
    loading = false;
  });

  let unreadCount = $derived(inbox.filter(m => !m.read_at).length);
</script>

<svelte:head><title>RUSA IMS — Dashboard</title></svelte:head>

<PageShell title="Mission Control" subtitle="Welcome to the RUSA Internal Management System">
  <div class="dashboard-grid">
    <div class="dash-card">
      <div class="dash-card-label">Logged In As</div>
      <div class="dash-card-value">{$session?.full_name}</div>
      <div class="dash-card-sub">{$session?.role_display_name}</div>
    </div>
    <div class="dash-card">
      <div class="dash-card-label">Unread Messages</div>
      <div class="dash-card-value" style="color: {unreadCount > 0 ? '#ffaa00' : '#00e5a0'}">{unreadCount}</div>
      <div class="dash-card-sub">in your inbox</div>
    </div>
    <div class="dash-card">
      <div class="dash-card-label">Access Tier</div>
      <div class="dash-card-value">Tier {$session?.tier}</div>
      <div class="dash-card-sub">{$session?.tier === 4 ? 'Administrator' : $session?.tier === 3 ? 'Director' : $session?.tier === 2 ? 'Staff Lead' : 'Staff'}</div>
    </div>
    <div class="dash-card">
      <div class="dash-card-label">System Status</div>
      <div class="dash-card-value" style="color: #00e5a0">ONLINE</div>
      <div class="dash-card-sub">All systems nominal</div>
    </div>
  </div>

  <div class="recent-messages">
    <h2 class="section-title">Recent Messages</h2>
    {#if loading}
      <p style="color: #4a5d82">Loading...</p>
    {:else if inbox.length === 0}
      <p style="color: #4a5d82">No messages in your inbox.</p>
    {:else}
      <div class="message-list">
        {#each inbox.slice(0, 5) as msg}
          <a href="/messages" class="msg-row" class:unread={!msg.read_at}>
            <span class="msg-from">{msg.from_name || msg.from_username || 'Unknown'}</span>
            <span class="msg-subject">{msg.subject}</span>
            <span class="msg-time">{msg.sent_at ? new Date(msg.sent_at).toLocaleDateString() : '—'}</span>
          </a>
        {/each}
      </div>
    {/if}
  </div>

  <div class="active-broadcasts">
    <h2 class="section-title" style="color: #ff4466;">Active Broadcasts</h2>
    {#if loading}
      <p style="color: #4a5d82">Loading...</p>
    {:else if broadcasts.length === 0}
      <p style="color: #4a5d82">No active broadcasts.</p>
    {:else}
      <div class="broadcast-list">
        {#each broadcasts.slice(0, 5) as b}
          <a href="/messages" class="broadcast-row" class:unread={!b.read_at}>
            <span class="broadcast-sender">{b.broadcast_sender || 'SYSTEM'}</span>
            <span class="broadcast-subject">{b.subject}</span>
            <span class="broadcast-time">{b.sent_at ? new Date(b.sent_at).toLocaleDateString() : '—'}</span>
          </a>
        {/each}
      </div>
    {/if}
  </div>

  <div class="schedule-widget">
    <h2 class="section-title">My Schedule Timeline</h2>
    {#if scheduleLoading}
      <p style="color: #4a5d82">Loading schedule...</p>
    {:else if scheduleItems.length === 0}
      <p style="color: #4a5d82">No upcoming assignments found.</p>
    {:else}
      <div class="shift-list">
        {#each scheduleItems.slice(0, MAX_DISPLAYED_SCHEDULE_ITEMS) as item}
          <div class="shift-row">
            <div class="shift-dept">{item.source}</div>
            <div class="shift-time">
              {#if item.kind === 'shift'}
                {formatDateTime(item.starts_at)} → {formatDateTime(item.ends_at)}
              {:else if item.kind === 'appointment'}
                {formatDateTime(item.starts_at)}
              {:else}
                Due {formatDateTime(item.due_at)}
              {/if}
            </div>
            <div class="shift-notes">
              <strong>{item.title}</strong>{item.status ? ` (${item.status})` : ''}
              {item.notes ? ` - ${item.notes}` : ''}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</PageShell>

<style>
  .dashboard-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-bottom: 2rem; }
  .dash-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.25rem 1.5rem; }
  .dash-card-label { font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; margin-bottom: 0.5rem; }
  .dash-card-value { font-family: 'Outfit', sans-serif; font-size: 1.75rem; font-weight: 600; color: #e8eeff; margin-bottom: 0.25rem; }
  .dash-card-sub { font-size: 0.75rem; color: #8fa3cc; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin-bottom: 1rem; }

  .recent-messages, .active-broadcasts { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; margin-bottom: 2rem; }
  .message-list, .broadcast-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .msg-row { display: grid; grid-template-columns: 150px 1fr 100px; gap: 1rem; padding: 0.75rem 1rem; background: #121b33; border: 1px solid #1e2d4a; border-radius: 4px; text-decoration: none; color: #8fa3cc; transition: all 0.2s; align-items: center; }
  .msg-row:hover { background: #1a2642; border-color: #2a3f6c; }
  .msg-row.unread { background: rgba(0, 212, 255, 0.05); border-color: rgba(0, 212, 255, 0.2); }
  .msg-row.unread .msg-subject { color: #e8eeff; font-weight: 500; }
  .msg-from { font-weight: 500; color: #c8d8f0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg-subject { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg-time { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-align: right; }

  .broadcast-row { display: grid; grid-template-columns: 150px 1fr 100px; gap: 1rem; padding: 0.75rem 1rem; background: #2a0812; border: 1px solid #ff4466; border-radius: 4px; text-decoration: none; color: #ff8899; transition: all 0.2s; align-items: center; }
  .broadcast-row:hover { background: #3a0a18; border-color: #ff6688; }
  .broadcast-row.unread { background: rgba(255, 68, 102, 0.15); border-color: #ff4466; box-shadow: 0 0 8px rgba(255, 68, 102, 0.3); }
  .broadcast-row.unread .broadcast-subject { color: #fff; font-weight: 600; }
  .broadcast-sender { font-weight: 700; color: #ff4466; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .broadcast-subject { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .broadcast-time { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-align: right; color: #ff8899; }
  .schedule-widget { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; margin-bottom: 2rem; }
  .shift-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .shift-row { display: grid; grid-template-columns: 100px 1.3fr 1fr; gap: 0.75rem; background: #121b33; border: 1px solid #1e2d4a; border-radius: 4px; padding: 0.75rem 1rem; }
  .shift-dept { color: #00d4ff; font-family: 'Space Mono', monospace; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.08em; }
  .shift-time { color: #c8d8f0; }
  .shift-notes { color: #8fa3cc; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
