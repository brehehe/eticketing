<script lang="ts">
  // Thin wrapper — re-export lucide-svelte icons with consistent sizing
  export let name: string;
  export let size: number = 16;
  export let strokeWidth: number = 1.75;
  export let className: string = '';
</script>

<!-- Dynamic icon via CSS mask — we use lucide-svelte components directly in usage -->
<!-- This component is a placeholder for consistent sizing API -->
<span
  class="icon {className}"
  style="--icon-size: {size}px;"
  aria-hidden="true"
  role="img"
>
  <slot />
</span>

<style>
  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-size, 16px);
    height: var(--icon-size, 16px);
    flex-shrink: 0;
  }
</style>
