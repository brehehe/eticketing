<script lang="ts">
  import { initials } from '$lib/utils/format';
  export let name: string = '';
  export let src: string | undefined = undefined;
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let color: string | undefined = undefined;

  const sizes = { xs: 24, sm: 28, md: 36, lg: 44, xl: 56 };
  const px = sizes[size];

  // Deterministic color from name
  const colors = [
    '#6366f1','#8b5cf6','#ec4899','#0ea5e9','#10b981','#f59e0b','#f43f5e'
  ];
  function colorFromName(n: string) {
    let hash = 0;
    for (let i = 0; i < n.length; i++) hash = n.charCodeAt(i) + ((hash << 5) - hash);
    return colors[Math.abs(hash) % colors.length];
  }

  $: bg = color ?? colorFromName(name);
  $: text = initials(name || '?');
  $: fontSize = px * 0.36 + 'px';
</script>

<span
  class="avatar avatar-{size}"
  style="width:{px}px;height:{px}px;background:{src ? 'transparent' : bg};font-size:{fontSize}"
  aria-label={name}
>
  {#if src}
    <img src={src} alt={name} width={px} height={px} />
  {:else}
    {text}
  {/if}
</span>

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-full);
    font-weight: 600;
    color: #fff;
    flex-shrink: 0;
    overflow: hidden;
    user-select: none;
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: inherit;
  }
</style>
