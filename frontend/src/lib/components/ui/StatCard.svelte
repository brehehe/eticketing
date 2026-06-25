<script lang="ts">
  export let label: string;
  export let value: string;
  export let sub: string = '';
  export let trend: number | undefined = undefined; // positive = up, negative = down
  export let color: string = 'var(--brand-500)';
  export let loading = false;
</script>

<div class="stat-card card fade-in">
  <div class="stat-header">
    <span class="stat-label">{label}</span>
    {#if trend !== undefined}
      <span class="trend" class:up={trend >= 0} class:down={trend < 0}>
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" aria-hidden="true">
          {#if trend >= 0}
            <path d="M6 2L10 8H2L6 2Z"/>
          {:else}
            <path d="M6 10L2 4H10L6 10Z"/>
          {/if}
        </svg>
        {Math.abs(trend).toFixed(1)}%
      </span>
    {/if}
  </div>

  {#if loading}
    <div class="skel" style="height:32px;width:60%;margin:6px 0;"></div>
    <div class="skel" style="height:14px;width:40%;"></div>
  {:else}
    <div class="stat-value" style="color:{color}">{value}</div>
    {#if sub}
      <div class="stat-sub">{sub}</div>
    {/if}
  {/if}

  <div class="stat-accent" style="background:{color}"></div>
</div>

<style>
  .stat-card {
    padding: 20px;
    position: relative;
    overflow: hidden;
    transition: transform var(--ease-fast), box-shadow var(--ease-fast);
  }
  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }
  .stat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .stat-label {
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-2);
  }
  .stat-value {
    font-size: 1.75rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1;
    margin-bottom: 4px;
  }
  .stat-sub {
    font-size: 0.8125rem;
    color: var(--text-2);
  }
  .trend {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: var(--r-full);
  }
  .trend.up   { color: var(--emerald); background: rgba(16,185,129,0.1); }
  .trend.down { color: var(--rose);    background: rgba(244,63,94,0.1); }
  .stat-accent {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 3px;
    opacity: 0.6;
    border-radius: 0 0 var(--r-lg) var(--r-lg);
  }
</style>
