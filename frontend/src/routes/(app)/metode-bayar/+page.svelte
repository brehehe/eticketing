<script lang="ts">
  export let data: any = {};
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import { toast } from '$lib/stores/toast';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { Plus, Edit2, Trash2, RefreshCw } from 'lucide-svelte';

  interface PaymentMethod {
    id: string;
    name: string;
    provider: string | null;
    fee_pct: number;
    fee_flat: number;
    auto_settlement: boolean;
    status: string;
    created_at?: string;
  }

  let methods: PaymentMethod[] = [];
  let loading = true;
  let modalOpen = false;
  let deleteOpen = false;
  let editTarget: PaymentMethod | null = null;
  let deleteTarget: PaymentMethod | null = null;
  let saving = false;

  let form = {
    name: '',
    provider: '',
    fee_pct: 0,
    fee_flat: 0,
    auto_settlement: true,
    status: 'active',
  };

  async function loadMethods() {
    loading = true;
    try {
      const res = await api.get<PaymentMethod[]>('/payment-methods');
      const d = (res as any)?.data;
      methods = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat metode pembayaran', e.message);
      methods = [];
    } finally {
      loading = false;
    }
  }

  onMount(loadMethods);

  function openNew() {
    editTarget = null;
    form = { name: '', provider: '', fee_pct: 0, fee_flat: 0, auto_settlement: true, status: 'active' };
    modalOpen = true;
  }

  function openEdit(m: PaymentMethod) {
    editTarget = m;
    form = {
      name: m.name,
      provider: m.provider ?? '',
      fee_pct: m.fee_pct,
      fee_flat: m.fee_flat,
      auto_settlement: m.auto_settlement,
      status: m.status,
    };
    modalOpen = true;
  }

  async function save() {
    if (!form.name.trim()) return;
    saving = true;
    try {
      const payload = {
        ...form,
        provider: form.provider || null,
        fee_pct: Number(form.fee_pct),
        fee_flat: Number(form.fee_flat),
      };
      if (editTarget) {
        await api.put(`/payment-methods/${editTarget.id}`, payload);
        toast.success('Metode diperbarui');
      } else {
        await api.post('/payment-methods', payload);
        toast.success('Metode ditambahkan');
      }
      modalOpen = false;
      await loadMethods();
    } catch (e: any) {
      toast.error('Gagal menyimpan', e.message);
    } finally {
      saving = false;
    }
  }

  async function deleteMethod() {
    if (!deleteTarget) return;
    saving = true;
    try {
      await api.delete(`/payment-methods/${deleteTarget.id}`);
      toast.success('Metode dihapus');
      deleteOpen = false;
      await loadMethods();
    } catch (e: any) {
      toast.error('Gagal menghapus', e.message);
    } finally {
      saving = false;
    }
  }

  const methodEmoji: Record<string, string> = {
    'Tunai': '\uD83D\uDCB5',
    'QRIS': '\uD83D\uDCF1',
    'Transfer Bank': '\uD83C\uDFE6',
    'Debit': '\uD83D\uDCB3',
    'E-Wallet': '\uD83D\uDCF2',
  };
</script>

<svelte:head><title>Metode Pembayaran \u2014 TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Metode Pembayaran</h1>
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">{methods.length} metode tersedia</p>
    </div>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary btn-sm" on:click={loadMethods} disabled={loading} aria-label="Refresh">
        <RefreshCw size={13} />
      </button>
      <button class="btn btn-primary" on:click={openNew}>
        <Plus size={15} /> Tambah Metode
      </button>
    </div>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">
      {#each Array(4) as _}<div class="skel" style="height:72px;"></div>{/each}
    </div>
  {:else if methods.length === 0}
    <div class="empty-state">Tidak ada metode pembayaran</div>
  {:else}
    <div class="methods-list">
      {#each methods as m (m.id)}
        <div class="method-card card">
          <div class="method-icon">
            <span style="font-size:1.5rem;">{methodEmoji[m.name] ?? '\uD83D\uDCB3'}</span>
          </div>
          <div class="method-info">
            <div style="display:flex;align-items:center;gap:8px;">
              <span style="font-weight:600;font-size:0.9375rem;">{m.name}</span>
              <Badge variant={m.status === 'active' ? 'success' : 'neutral'} dot>
                {m.status === 'active' ? 'Aktif' : 'Nonaktif'}
              </Badge>
            </div>
            <div style="display:flex;align-items:center;gap:12px;margin-top:4px;flex-wrap:wrap;">
              {#if m.provider}
                <span style="font-size:0.8125rem;color:var(--text-2);">Provider: <strong>{m.provider}</strong></span>
              {/if}
              {#if m.fee_pct > 0}
                <span style="font-size:0.8125rem;color:var(--text-2);">Fee: <strong>{m.fee_pct}%</strong></span>
              {:else if m.fee_flat > 0}
                <span style="font-size:0.8125rem;color:var(--text-2);">Fee: <strong>Rp{m.fee_flat.toLocaleString('id-ID')}</strong></span>
              {:else}
                <span style="font-size:0.8125rem;color:var(--text-3);">Tanpa fee</span>
              {/if}
              <Badge variant={m.auto_settlement ? 'info' : 'neutral'} size="sm">
                {m.auto_settlement ? 'Auto Settlement' : 'Manual'}
              </Badge>
            </div>
          </div>
          <div style="display:flex;gap:4px;margin-left:auto;flex-shrink:0;">
            <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEdit(m)} aria-label="Edit {m.name}">
              <Edit2 size={14} />
            </button>
            <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => { deleteTarget=m; deleteOpen=true; }} aria-label="Hapus {m.name}">
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add/Edit Modal -->
<Modal bind:open={modalOpen} title={editTarget ? 'Edit Metode' : 'Tambah Metode Bayar'} size="md">
  <div class="modal-body">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
      <div class="field" style="grid-column:1/-1;">
        <label class="label" for="pm-name">Nama Metode</label>
        <input id="pm-name" class="input" bind:value={form.name} placeholder="Contoh: QRIS, Transfer, dll" />
      </div>
      <div class="field">
        <label class="label" for="pm-provider">Provider</label>
        <input id="pm-provider" class="input" bind:value={form.provider} placeholder="Midtrans, BCA, dll" />
      </div>
      <div class="field">
        <label class="label" for="pm-fee">Fee (%)</label>
        <input id="pm-fee" class="input" type="number" min="0" step="0.01" bind:value={form.fee_pct} />
      </div>
      <div class="field">
        <label class="label" for="pm-flat">Fee Flat (Rp)</label>
        <input id="pm-flat" class="input" type="number" min="0" bind:value={form.fee_flat} />
      </div>
      <div class="field">
        <label class="label" for="pm-status">Status</label>
        <Select
          id="pm-status"
          bind:value={form.status}
          options={[
            { value: 'active',   label: 'Aktif'    },
            { value: 'inactive', label: 'Nonaktif' },
          ]}
        />
      </div>
    </div>
    <div style="padding:14px;background:var(--bg-subtle);border-radius:var(--r-md);margin-top:8px;">
      <label style="display:flex;align-items:center;gap:8px;font-size:0.875rem;color:var(--text-2);cursor:pointer;user-select:none;">
        <input type="checkbox" bind:checked={form.auto_settlement} style="accent-color:var(--brand-500);" />
        Auto Settlement (konfirmasi otomatis setelah pembayaran)
      </label>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (modalOpen=false)}>Batal</button>
    <button class="btn btn-primary" on:click={save} disabled={saving || !form.name.trim()}>
      {#if saving}<Spinner size={14} color="#fff" />{/if}
      {editTarget ? 'Simpan' : 'Tambah'}
    </button>
  </div>
</Modal>

<!-- Delete -->
<Modal bind:open={deleteOpen} title="Hapus Metode" size="sm">
  <div class="modal-body">
    <p>Yakin ingin menghapus metode <strong>{deleteTarget?.name}</strong>?</p>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (deleteOpen=false)}>Batal</button>
    <button class="btn btn-danger" on:click={deleteMethod} disabled={saving}>
      {#if saving}<Spinner size={14} color="#fff" />{/if} Hapus
    </button>
  </div>
</Modal>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1400px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .methods-list { display:flex;flex-direction:column;gap:10px; }
  .method-card {
    display:flex;
    align-items:center;
    gap:16px;
    padding:16px 20px;
    transition:box-shadow var(--ease-fast);
  }
  .method-card:hover { box-shadow:var(--shadow-md); }
  .method-icon {
    width:48px;height:48px;
    border-radius:var(--r-lg);
    background:var(--bg-subtle);
    display:flex;align-items:center;justify-content:center;
    flex-shrink:0;
  }
  .method-info { flex:1;min-width:0; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
</style>
