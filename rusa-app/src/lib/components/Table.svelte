<script lang="ts">
  let {
    columns = [] as { key: string; label: string; width?: string }[],
    rows = [] as any[],
    onRowClick = undefined as ((row: any) => void) | undefined
  } = $props();
</script>

<div class="table-wrapper">
  <table class="rusa-table">
    <thead>
      <tr>
        {#each columns as col}
          <th style={col.width ? `width: ${col.width}` : ''}>{col.label}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#if rows.length === 0}
        <tr>
          <td colspan={columns.length} class="empty-state">No records found.</td>
        </tr>
      {:else}
        {#each rows as row}
          <tr class:clickable={!!onRowClick} onclick={() => onRowClick?.(row)}>
            {#each columns as col}
              <td>{row[col.key] ?? '—'}</td>
            {/each}
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-wrapper { overflow-x: auto; border-radius: 6px; border: 1px solid #1e2d4a; }
  .rusa-table { width: 100%; border-collapse: collapse; font-family: 'Outfit', sans-serif; font-size: 0.875rem; }
  th {
    background: #080d1a; color: #8fa3cc;
    font-family: 'Space Mono', monospace; font-size: 0.65rem;
    text-transform: uppercase; letter-spacing: 0.08em;
    padding: 0.75rem 1rem; text-align: left; border-bottom: 1px solid #1e2d4a;
  }
  td { padding: 0.75rem 1rem; color: #e8eeff; border-bottom: 1px solid #0d1528; }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: rgba(61,127,255,0.04); }
  tr.clickable { cursor: pointer; }
  .empty-state { text-align: center; color: #4a5d82; padding: 2rem; }
</style>
