<script lang="ts">
  export let variant: 'brand' | 'success' | 'warning' | 'danger' | 'info' | 'neutral' = 'neutral';
  export let dot = false;
  export let pulse = false;
  export let size: 'sm' | 'md' = 'md';
</script>

<span class="badge badge-{variant}" class:badge-sm={size === 'sm'}>
  {#if dot}
    <span
      class="dot dot-{variant === 'neutral' ? 'gray' : variant === 'brand' ? 'brand' : variant}"
      class:dot-pulse={pulse}
    />
  {/if}
  <slot />
</span>

<style>
  .badge-sm { height: 18px; padding: 0 6px; font-size: 0.625rem; }
</style>
