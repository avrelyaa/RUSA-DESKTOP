<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import { session } from '$lib/stores/auth';
  import { messageApi } from '$lib/api';
  import { onMount } from 'svelte';

  let inbox: any[] = $state([]);
  let loading = $state(true);

  onMount(async () => {
    const s = $session;
    if (!s) return;
    try { inbox = await messageApi.getInbox(s.token); } catch {}
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
</PageShell>

<style>
  .dashboard-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-bottom: 2rem; }
  .dash-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.25rem 1.5rem; }
  .dash-card-label { font-family: 'Space Mono', monospace; font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.08em; color: #4a5d82; margin-bottom: 0.5rem; }
  .dash-card-value { font-size: 1.4rem; font-weight: 600; color: #e8eeff; margin-bottom: 0.25rem; }
  .dash-card-sub { font-size: 0.75rem; color: #8fa3cc; }
  .section-title { font-family: 'Space Mono', monospace; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: #8fa3cc; margin-bottom: 1rem; }
  .recent-messages { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.25rem; }
  .message-list { display: flex; flex-direction: column; gap: 0.25rem; }
  .msg-row {
    display: grid; grid-template-columns: 150px 1fr 100px;
    gap: 1rem; padding: 0.625rem 0.75rem; border-radius: 4px;
    text-decoration: none; color: #8fa3cc; font-size: 0.85rem;
    transition: background 0.15s;
  }
  .msg-row:hover { background: rgba(61,127,255,0.06); }
  .msg-row.unread { color: #e8eeff; font-weight: 500; }
  .msg-from { color: #00d4ff; font-size: 0.8rem; }
  .msg-time { font-size: 0.75rem; text-align: right; color: #4a5d82; }
</style>
