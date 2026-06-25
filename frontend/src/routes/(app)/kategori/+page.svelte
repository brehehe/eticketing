<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { categoriesApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { Plus, Edit2, Trash2, Tag, RefreshCw, GripVertical } from 'lucide-svelte';

  interface Category {
    id: string;
    name: string;
    slug: string;
    icon: string | null;
    sort_order: number;
    created_at?: string;
  }

  let categories: Category[] = [];
  let loading = true;
  let modalOpen = false;
  let deleteOpen = false;
  let editTarget: Category | null = null;
  let deleteTarget: Category | null = null;
  let saving = false;

  let categoryForm = { name: '', slug: '', icon: '', sort_order: 0 };

  const iconOptions = [
    { value: 'Ticket',       label: '\uD83C\uDFA1 Wahana'   },
    { value: 'UtensilsCrossed', label: '\uD83C\uDF7D\uFE0F Makanan'  },
    { value: 'Coffee',       label: '\u2615 Minuman'  },
    { value: 'ShoppingBag',  label: '\uD83D\uDECD\uFE0F Souvenir' },
    { value: 'Star',         label: '\u2B50 Unggulan'  },
    { value: 'Zap',          label: '\u26A1 Promo'    },
    { value: 'Gift',         label: '\uD83C\uDF81 Hadiah'   },
    { value: 'Package',      label: '\uD83D\uDCE6 Lainnya'  },
  ];

  async function loadCategories() {
    loading = true;
    try {
      const res = await categoriesApi.list();
      const d = (res as any)?.data;
      categories = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat kategori', e.message);
    } finally {
      loading = false;
    }
  }

  onMount(loadCategories);

  function openNew() {
    editTarget = null;
    categoryForm = { name: '', slug: '', icon: 'Package', sort_order: categories.length };
    modalOpen = true;
  }

  function openEdit(c: Category) {
    editTarget = c;
    categoryForm = { name: c.name, slug: c.slug, icon: c.icon ?? '', sort_order: c.sort_order };
    modalOpen = true;
  }

  // Auto-generate slug from name
  function onNameInput() {
    if (!editTarget) {
      categoryForm.slug = categoryForm.name
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/(^-|-$)/g, '');
    }
  }

  async function save() {
    if (!categoryForm.name.trim()) return;
    saving = true;
    try {
      const payload = {
        name: categoryForm.name,
        slug: categoryForm.slug || categoryForm.name.toLowerCase().replace(/\s+/g, '-'),
        icon: categoryForm.icon || null,
        sort_order: Number(categoryForm.sort_order),
      };
      if (editTarget) {
        await categoriesApi.update(editTarget.id, payload);
        toast.success('Kategori diperbarui');
      } else {
        await categoriesApi.create(payload);
        toast.success('Kategori ditambahkan');
      }
      modalOpen = false;
      await loadCategories();
    } catch (e: any) {
      toast.error('Gagal menyimpan', e.message);
    } finally {
      saving = false;
    }
  }

  async function deleteCategory() {
    if (!deleteTarget) return;
    saving = true;
    try {
      await categoriesApi.delete(deleteTarget.id);
      toast.success('Kategori dihapus');
      deleteOpen = false;
      await loadCategories();
    } catch (e: any) {
      toast.error('Gagal menghapus', e.message);
    } finally {
      saving = false;
    }
  }

  const emojiMap: Record<string, string> = {
    Ticket: '\uD83C\uDFA1', UtensilsCrossed: '\uD83C\uDF7D\uFE0F', Coffee: '\u2615',
    ShoppingBag: '\uD83D\uDECD\uFE0F', Star: '\u2B50', Zap: '\u26A1',
    Gift: '\uD83C\uDF81', Package: '\uD83D\uDCE6',
  };
</script>

<svelte:head><title>Kategori Produk \u2014 TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Kategori Produk</h1>
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">{categories.length} kategori</p>
    </div>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary btn-sm" on:click={loadCategories} disabled={loading}><RefreshCw size={13} /></button>
      <button class="btn btn-primary" on:click={openNew}><Plus size={15} /> Tambah Kategori</button>
    </div>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">
      {#each Array(4) as _}<div class="skel" style="height:64px;"></div>{/each}
    </div>
  {:else if categories.length === 0}
    <div class="empty-state">Belum ada kategori</div>
  {:else}
    <div class="cat-grid">
      {#each categories as c (c.id)}
        <div class="cat-card card">
          <div class="cat-emoji">{emojiMap[c.icon ?? ''] ?? '\uD83D\uDCE6'}</div>
          <div class="cat-info">
            <span class="cat-name">{c.name}</span>
            <div style="display:flex;align-items:center;gap:6px;margin-top:3px;">
              <span class="mono" style="font-size:0.75rem;color:var(--text-3);">{c.slug}</span>
              <Badge variant="neutral" size="sm">urutan {c.sort_order}</Badge>
            </div>
          </div>
          <div style="display:flex;gap:4px;margin-left:auto;">
            <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEdit(c)} aria-label="Edit {c.name}"><Edit2 size={14} /></button>
            <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => { deleteTarget=c; deleteOpen=true; }} aria-label="Hapus {c.name}"><Trash2 size={14} /></button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Modal bind:open={modalOpen} title={editTarget ? 'Edit Kategori' : 'Tambah Kategori'} size="sm">
  <div class="modal-body">
    <div style="display:flex;flex-direction:column;gap:14px;">
      <div class="field">
        <label class="label" for="cat-name">Nama Kategori</label>
        <input id="cat-name" class="input" bind:value={categoryForm.name} on:input={onNameInput} placeholder="Contoh: Wahana" />
      </div>
      <div class="field">
        <label class="label" for="cat-slug">Slug (URL)</label>
        <input id="cat-slug" class="input" bind:value={categoryForm.slug} placeholder="wahana" />
        <span class="field-hint">Huruf kecil, tanpa spasi. Contoh: wahana-air</span>
      </div>
      <div class="field">
        <label class="label" for="cat-icon">Icon</label>
        <select id="cat-icon" class="input" bind:value={categoryForm.icon}>
          {#each iconOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label class="label" for="cat-order">Urutan Tampil</label>
        <input id="cat-order" class="input" type="number" min="0" bind:value={categoryForm.sort_order} />
        <span class="field-hint">Angka kecil tampil lebih dulu</span>
      </div>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (modalOpen=false)}>Batal</button>
    <button class="btn btn-primary" on:click={save} disabled={saving || !categoryForm.name.trim()}>
      {#if saving}<Spinner size={14} color="#fff" />{/if}
      {editTarget ? 'Simpan' : 'Tambah'}
    </button>
  </div>
</Modal>

<Modal bind:open={deleteOpen} title="Hapus Kategori" size="sm">
  <div class="modal-body">
    <p>Yakin ingin menghapus kategori <strong>{deleteTarget?.name}</strong>?</p>
    <p style="font-size:0.875rem;color:var(--text-2);margin-top:8px;">Kategori yang masih digunakan produk tidak dapat dihapus.</p>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (deleteOpen=false)}>Batal</button>
    <button class="btn btn-danger" on:click={deleteCategory} disabled={saving}>
      {#if saving}<Spinner size={14} color="#fff" />{/if} Hapus
    </button>
  </div>
</Modal>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:800px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .cat-grid { display:flex;flex-direction:column;gap:10px; }
  .cat-card {
    display:flex;align-items:center;gap:14px;padding:14px 18px;
    transition:box-shadow var(--ease-fast);
  }
  .cat-card:hover { box-shadow:var(--shadow-md); }
  .cat-emoji {
    width:44px;height:44px;border-radius:var(--r-lg);
    background:var(--brand-50);display:flex;align-items:center;
    justify-content:center;font-size:1.375rem;flex-shrink:0;
  }
  [data-theme='dark'] .cat-emoji { background:rgba(99,102,241,0.12); }
  .cat-info { flex:1;min-width:0; }
  .cat-name { font-size:0.9375rem;font-weight:600;color:var(--text-1); }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
</style>
