<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { X } from 'lucide-svelte';

  export let open = false;
  export let title: string = '';
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let closable = true;

  const dispatch = createEventDispatcher<{ close: void }>();

  function close() {
    if (!closable) return;
    open = false;
    dispatch('close');
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  const sizeMap = { sm: 'modal-sm', md: '', lg: 'modal-lg', xl: 'modal-xl' };
</script>

<svelte:window on:keydown={onKeydown} />

{#if open}
  <div
    class="modal-backdrop"
    on:click|self={close}
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <div class="modal {sizeMap[size]}">
      {#if title}
        <div class="modal-header">
          <h3 style="font-size:1rem; font-weight:600;">{title}</h3>
          {#if closable}
            <button class="btn btn-ghost btn-icon btn-sm" on:click={close} aria-label="Tutup">
              <X size={16} />
            </button>
          {/if}
        </div>
      {/if}
      <slot />
    </div>
  </div>
{/if}
