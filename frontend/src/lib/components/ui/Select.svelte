<script lang="ts">
  import { createEventDispatcher, onDestroy } from 'svelte';
  import { ChevronDown, Check } from 'lucide-svelte';

  export let id: string = '';
  export let value: any = '';
  export let options: { value: any; label: string }[] = [];
  export let placeholder: string = 'Pilih...';
  export let disabled: boolean = false;

  const dispatch = createEventDispatcher<{ change: any }>();

  let open = false;
  let triggerEl: HTMLButtonElement;
  let dropdownEl: HTMLDivElement;
  let dropdownStyle = '';

  $: selectedLabel = options.find(o => o.value === value)?.label ?? placeholder;

  function toggle() {
    if (disabled) return;
    if (!open) {
      open = true;
      positionDropdown();
    } else {
      open = false;
    }
  }

  function positionDropdown() {
    if (!triggerEl) return;
    const rect = triggerEl.getBoundingClientRect();
    const spaceBelow = window.innerHeight - rect.bottom;
    const dropH = Math.min(options.length * 40 + 8, 240);
    const showAbove = spaceBelow < dropH + 8 && rect.top > dropH + 8;

    dropdownStyle = [
      `position:fixed`,
      `left:${rect.left}px`,
      `width:${rect.width}px`,
      `z-index:9999`,
      showAbove
        ? `bottom:${window.innerHeight - rect.top + 4}px`
        : `top:${rect.bottom + 4}px`,
    ].join(';');
  }

  function select(opt: { value: any; label: string }) {
    value = opt.value;
    open = false;
    dispatch('change', value);
  }

  function handleOutside(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node;
    if (triggerEl?.contains(target) || dropdownEl?.contains(target)) return;
    open = false;
  }

  function handleScroll() { if (open) positionDropdown(); }
  function handleResize() { if (open) positionDropdown(); }

  onDestroy(() => { open = false; });
</script>

<svelte:window
  on:mousedown={handleOutside}
  on:scroll|capture={handleScroll}
  on:resize={handleResize}
/>

<div class="custom-select" class:disabled>
  <button
    {id}
    type="button"
    class="cs-trigger input"
    class:open
    {disabled}
    bind:this={triggerEl}
    on:click={toggle}
    aria-haspopup="listbox"
    aria-expanded={open}
  >
    <span class="cs-label" class:placeholder={!options.find(o => o.value === value)}>
      {selectedLabel}
    </span>
    <ChevronDown size={16} class="cs-chevron {open ? 'rotated' : ''}" />
  </button>

  {#if open}
    <div
      class="cs-dropdown"
      style={dropdownStyle}
      role="listbox"
      bind:this={dropdownEl}
    >
      {#each options as opt}
        <button
          type="button"
          class="cs-option"
          class:selected={opt.value === value}
          role="option"
          aria-selected={opt.value === value}
          on:click={() => select(opt)}
        >
          <span>{opt.label}</span>
          {#if opt.value === value}
            <Check size={14} />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .custom-select { position: relative; width: 100%; }
  .custom-select.disabled { opacity: 0.5; pointer-events: none; }

  .cs-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    cursor: pointer;
    text-align: left;
    width: 100%;
    padding-right: 10px;
  }
  .cs-trigger.open {
    border-color: var(--brand-500);
    box-shadow: 0 0 0 3px rgba(139,92,246,0.15);
    outline: none;
  }

  .cs-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .cs-label.placeholder { color: var(--text-3); }

  :global(.cs-chevron) {
    flex-shrink: 0;
    color: var(--text-3);
    transition: transform 200ms ease;
  }
  :global(.cs-chevron.rotated) { transform: rotate(180deg); }

  .cs-dropdown {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-xl);
    overflow-y: auto;
    max-height: 240px;
    padding: 4px;
    animation: dropIn 120ms ease;
  }
  @keyframes dropIn {
    from { opacity: 0; transform: translateY(-4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .cs-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 9px 12px;
    border-radius: var(--r-md);
    font-size: 0.875rem;
    color: var(--text-1);
    cursor: pointer;
    transition: background var(--ease-fast);
    text-align: left;
    background: transparent;
    border: none;
  }
  .cs-option:hover { background: var(--bg-subtle); }
  .cs-option.selected {
    background: rgba(139,92,246,0.08);
    color: var(--brand-500);
    font-weight: 600;
  }
  .cs-option.selected :global(svg) { color: var(--brand-500); }
</style>
