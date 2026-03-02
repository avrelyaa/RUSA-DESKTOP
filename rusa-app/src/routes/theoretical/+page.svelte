<script lang="ts">
  import PageShell from '$lib/components/PageShell.svelte';
  import { session } from '$lib/stores/auth';
  import { showToast } from '$lib/stores/toast';

  // Theoretical sciences page — placeholder for mathematician / artificer roles
  // Tasks are managed via messaging and governance systems
</script>

<svelte:head><title>RUSA IMS — Theoretical Sciences</title></svelte:head>

<PageShell title="Theoretical Sciences" subtitle="Mathematical research and theoretical frameworks">
  <div class="info-grid">
    <div class="info-card">
      <div class="info-icon">∑</div>
      <h3 class="info-title">Mathematical Research</h3>
      <p class="info-desc">Submit theoretical research proposals and mathematical models through the governance system or via internal messaging to directors.</p>
    </div>
    <div class="info-card">
      <div class="info-icon">∞</div>
      <h3 class="info-title">Data Collaboration</h3>
      <p class="info-desc">Request datasets from the Data Services team to support theoretical work. Use the Data Request page to submit formal requests.</p>
    </div>
    <div class="info-card">
      <div class="info-icon">⊕</div>
      <h3 class="info-title">Cross-Subsystem Support</h3>
      <p class="info-desc">Theoretical sciences collaborates across all RUSA subsystems. Use the messaging system to coordinate with researchers and engineers.</p>
    </div>
  </div>

  <div class="actions">
    <a href="/data-request" class="action-btn">Request Data</a>
    <a href="/messages" class="action-btn action-btn--secondary">Send Message</a>
    <a href="/governance" class="action-btn action-btn--secondary">View Governance</a>
  </div>
</PageShell>

<style>
  .info-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.25rem; margin-bottom: 2rem; }
  .info-card { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 8px; padding: 1.5rem; text-align: center; }
  .info-icon { font-size: 2rem; margin-bottom: 1rem; color: #3d7fff; }
  .info-title { font-family: 'Space Mono', monospace; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.08em; color: #e8eeff; margin-bottom: 0.75rem; }
  .info-desc { font-size: 0.85rem; color: #8fa3cc; line-height: 1.6; }
  .actions { display: flex; gap: 1rem; }
  .action-btn { background: linear-gradient(135deg, #3d7fff, #00d4ff); border: none; border-radius: 4px; color: #05070f; cursor: pointer; font-family: 'Space Mono', monospace; font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; padding: 0.75rem 1.5rem; text-decoration: none; }
  .action-btn--secondary { background: none; border: 1px solid #1e2d4a; color: #8fa3cc; }
  .action-btn--secondary:hover { border-color: #3d7fff; color: #3d7fff; }
</style>
