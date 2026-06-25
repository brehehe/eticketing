<script lang="ts">
  import { toast, type Toast } from '$lib/stores/toast';
  import { CheckCircle, XCircle, AlertTriangle, Info, X } from 'lucide-svelte';

  const icons = {
    success: CheckCircle,
    error:   XCircle,
    warning: AlertTriangle,
    info:    Info,
  };

  const colors = {
    success: 'var(--emerald)',
    error:   'var(--rose)',
    warning: 'var(--amber)',
    info:    'var(--sky)',
  };

  let toasts: Toast[] = [];
  toast.subscribe(t => { toasts = t; });
</script>

<div class="toast-stack" aria-live="polite" aria-label="Notifikasi">
  {#each toasts as t (t.id)}
    <div class="toast" role="alert">
      <span style="color: {colors[t.type]}; flex-shrink:0; display:flex;">
        <svelte:component this={icons[t.type]} size={18} />
      </span>
      <div style="flex:1; min-width:0;">
        <p style="font-weight:600; font-size:0.875rem; line-height:1.4;">{t.title}</p>
        {#if t.message}
          <p style="font-size:0.8125rem; color:var(--text-2); margin-top:2px;">{t.message}</p>
        {/if}
      </div>
      <button
        class="btn btn-ghost btn-icon btn-sm"
        style="flex-shrink:0;"
        on:click={() => toast.remove(t.id)}
        aria-label="Tutup notifikasi"
      >
        <X size={14} />
      </button>
    </div>
  {/each}
</div>
