<script lang="ts">
  export let size: number = 18;
  export let color: string = 'currentColor';
</script>

<svg
  class="spinner"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  aria-hidden="true"
>
  <circle cx="12" cy="12" r="9" stroke={color} stroke-width="2.5" stroke-linecap="round" stroke-dasharray="40 20" />
</svg>

<style>
  .spinner {
    animation: spin 0.75s linear infinite;
    flex-shrink: 0;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
