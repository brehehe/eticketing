<script lang="ts">
  export let data: any = {};
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { currency } from '$lib/utils/format';
  import { productsApi, categoriesApi, variantsApi, investorsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { Product } from '$lib/types';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { Plus, Search, Edit2, Trash2, Package, ChevronDown, ChevronUp, X } from 'lucide-svelte';

  let products: Product[] = [];
  let categories: any[] = [];
  let investors: any[] = [];
  let loading = true;
  let total = 0;
  let searchQuery = '';
  let filterStatus = 'all';
  let modalOpen = false;
  let deleteOpen = false;
  let variantModalOpen = false;
  let editTarget: Product | null = null;
  let deleteTarget: Product | null = null;
  let saving = false;
  let expandedProduct: string | null = null;
  let productVariants: Record<string, any[]> = {};
  let loadingVariants: Record<string, boolean> = {};

  // Product form
  let form = {
    name: '', sku: '', category_id: '', price: 0, investor_price: 0, investor_id: '',
    has_variants: false, ticket_required: true,
    print_enabled: true, scan_enabled: true,
    status: 'active' as 'active' | 'inactive' | 'draft',
    quota: undefined as number | undefined,
  };

  // Variant form
  let variantForm = { name: '', sku: '', price: 0, investor_price: 0, quota: undefined as number | undefined, status: 'active' };
  let editVariantTarget: any | null = null;
  let variantProductId = '';

  async function loadProducts() {
    loading = true;
    try {
      const params: Record<string, string> = { per_page: '200' };
      if (filterStatus !== 'all') params.status = filterStatus;
      const [prodRes, catRes, invRes] = await Promise.all([
        productsApi.list(params),
        categoriesApi.list(),
        investorsApi.list(),
      ]);
      const d = (prodRes as any)?.data;
      products = Array.isArray(d) ? d : [];
      total = (prodRes as any)?.total ?? products.length;
      const cd = (catRes as any)?.data;
      categories = Array.isArray(cd) ? cd : [];
      const id_res = (invRes as any)?.data;
      investors = Array.isArray(id_res) ? id_res : [];
    } catch (e: any) {
      toast.error('Gagal memuat produk', e.message);
    } finally {
      loading = false;
    }
  }

  onMount(loadProducts);

  let timer: ReturnType<typeof setTimeout>;
  function onSearch() { clearTimeout(timer); timer = setTimeout(loadProducts, 400); }

  $: filteredProducts = products.filter(p => {
    const q = searchQuery.toLowerCase();
    return (!q || p.name.toLowerCase().includes(q) || p.sku.toLowerCase().includes(q)) &&
           (filterStatus === 'all' || p.status === filterStatus);
  });

  function openNew() {
    editTarget = null;
    form = { name:'', sku:'', category_id:'', price:0, investor_price:0, investor_id:'', has_variants:false, ticket_required:true, print_enabled:true, scan_enabled:true, status:'active', quota:undefined };
    modalOpen = true;
  }

  function openEdit(p: Product) {
    editTarget = p;
    form = { name:p.name, sku:p.sku, category_id:p.category_id ?? '', price:p.price, investor_price:p.investor_price ?? 0, investor_id:p.investor_id ?? '', has_variants:p.has_variants, ticket_required:p.ticket_required, print_enabled:p.print_enabled, scan_enabled:p.scan_enabled, status:p.status, quota:p.quota };
    modalOpen = true;
  }

  async function save() {
    if (!form.name || !form.sku) return;
    saving = true;
    try {
      const payload = {
        ...form,
        price: Number(form.price),
        investor_price: Number(form.investor_price),
        investor_id: form.investor_id || null,
        quota: form.quota ? Number(form.quota) : null,
        category_id: form.category_id || null
      };
      if (editTarget) {
        await productsApi.update(editTarget.id, payload);
        toast.success('Produk diperbarui');
      } else {
        await productsApi.create(payload);
        toast.success('Produk ditambahkan');
      }
      modalOpen = false;
      await loadProducts();
    } catch (e: any) {
      toast.error('Gagal menyimpan', e.message);
    } finally { saving = false; }
  }

  async function deleteProduct() {
    if (!deleteTarget) return;
    saving = true;
    try {
      await productsApi.delete(deleteTarget.id);
      toast.success('Produk dihapus');
      deleteOpen = false;
      await loadProducts();
    } catch (e: any) {
      toast.error('Gagal menghapus', e.message);
    } finally { saving = false; }
  }

  // Variants
  async function toggleVariants(productId: string) {
    if (expandedProduct === productId) {
      expandedProduct = null;
      return;
    }
    expandedProduct = productId;
    if (!productVariants[productId]) {
      loadingVariants = { ...loadingVariants, [productId]: true };
      try {
        const res = await variantsApi.list(productId);
        const d = (res as any)?.data;
        productVariants = { ...productVariants, [productId]: Array.isArray(d) ? d : [] };
      } catch (e: any) {
        toast.error('Gagal memuat varian', e.message);
      } finally {
        loadingVariants = { ...loadingVariants, [productId]: false };
      }
    }
  }

  function openNewVariant(productId: string) {
    variantProductId = productId;
    editVariantTarget = null;
    variantForm = { name: '', sku: '', price: 0, investor_price: 0, quota: undefined, status: 'active' };
    variantModalOpen = true;
  }

  function openEditVariant(productId: string, v: any) {
    variantProductId = productId;
    editVariantTarget = v;
    variantForm = { name: v.name, sku: v.sku, price: v.price, investor_price: v.investor_price ?? 0, quota: v.quota, status: v.status };
    variantModalOpen = true;
  }

  async function saveVariant() {
    if (!variantForm.name || !variantForm.sku) return;
    saving = true;
    try {
      const payload = {
        ...variantForm,
        price: Number(variantForm.price),
        investor_price: Number(variantForm.investor_price),
        quota: variantForm.quota ? Number(variantForm.quota) : null
      };
      if (editVariantTarget) {
        await variantsApi.update(variantProductId, editVariantTarget.id, payload);
        toast.success('Varian diperbarui');
      } else {
        await variantsApi.create(variantProductId, payload);
        toast.success('Varian ditambahkan');
      }
      variantModalOpen = false;
      // Reload variants for this product
      delete productVariants[variantProductId];
      productVariants = { ...productVariants };
      await toggleVariants(variantProductId);
      await loadProducts();
    } catch (e: any) {
      toast.error('Gagal menyimpan varian', e.message);
    } finally { saving = false; }
  }

  async function deleteVariant(productId: string, variantId: string, variantName: string) {
    if (!confirm(`Hapus varian "${variantName}"?`)) return;
    try {
      await variantsApi.delete(productId, variantId);
      toast.success('Varian dihapus');
      delete productVariants[productId];
      productVariants = { ...productVariants };
      await toggleVariants(productId);
      await loadProducts();
    } catch (e: any) {
      toast.error('Gagal menghapus varian', e.message);
    }
  }

  const catName = (id: string) => categories.find(c => c.id === id)?.name ?? '-';
</script>

<svelte:head><title>Produk \u2014 TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Produk</h1>
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">{total} produk terdaftar</p>
    </div>
    <button class="btn btn-primary" on:click={openNew}><Plus size={15} /> Tambah Produk</button>
  </div>

  <div class="filters">
    <div class="search-wrap">
      <Search size={15} style="color:var(--text-3);" />
      <input class="search-input" type="search" bind:value={searchQuery} on:input={onSearch} placeholder="Cari nama / SKU..." />
    </div>
    <select class="input input-sm" style="width:140px;" bind:value={filterStatus} on:change={loadProducts}>
      <option value="all">Semua Status</option>
      <option value="active">Aktif</option>
      <option value="inactive">Nonaktif</option>
      <option value="draft">Draft</option>
    </select>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">
      {#each Array(5) as _}<div class="skel" style="height:52px;"></div>{/each}
    </div>
  {:else if filteredProducts.length === 0}
    <div class="empty-state">Tidak ada produk ditemukan</div>
  {:else}
    <div class="products-list">
      {#each filteredProducts as p (p.id)}
        <div class="product-row card">
          <!-- Main row -->
          <div class="product-main">
            <div class="p-icon"><Package size={16} style="color:var(--brand-500);" /></div>
            <div class="p-info">
              <span style="font-weight:600;">{p.name}</span>
              <div style="display:flex;align-items:center;gap:6px;margin-top:2px;flex-wrap:wrap;">
                <span class="mono" style="font-size:0.75rem;color:var(--text-3);">{p.sku}</span>
                {#if p.category_id}<span style="font-size:0.75rem;color:var(--text-3);">&bull; {catName(p.category_id)}</span>{/if}
              </div>
            </div>
            <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap;">
              <div style="display:flex;flex-direction:column;align-items:flex-end;">
                <span style="font-weight:700;color:var(--brand-600);white-space:nowrap;">{currency(p.price)}</span>
                <span style="font-size:0.75rem;color:var(--text-3);white-space:nowrap;">HPP: {currency(p.investor_price)}</span>
              </div>
              {#if p.quota !== undefined && p.quota !== null}
                <span style="font-size:0.8125rem;color:var(--text-2);">Kuota: {(p.quota ?? 0) - p.quota_used}/{p.quota}</span>
              {/if}
              {#if p.has_variants}
                <Badge variant="info" size="sm">Varian</Badge>
              {/if}
              {#if p.ticket_required}<Badge variant="brand" size="sm">Tiket</Badge>{/if}
              <Badge variant={p.status === 'active' ? 'success' : p.status === 'draft' ? 'warning' : 'neutral'} dot size="sm">
                {p.status === 'active' ? 'Aktif' : p.status === 'draft' ? 'Draft' : 'Nonaktif'}
              </Badge>
            </div>
            <div style="display:flex;gap:4px;flex-shrink:0;">
              {#if p.has_variants || expandedProduct === p.id}
                <button
                  class="btn btn-ghost btn-sm"
                  style="gap:4px;font-size:0.8125rem;"
                  on:click={() => toggleVariants(p.id)}
                >
                  {#if expandedProduct === p.id}
                    <ChevronUp size={13} /> Varian
                  {:else}
                    <ChevronDown size={13} /> Varian
                  {/if}
                </button>
              {/if}
              <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEdit(p)} aria-label="Edit"><Edit2 size={14} /></button>
              <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => { deleteTarget=p; deleteOpen=true; }} aria-label="Hapus"><Trash2 size={14} /></button>
            </div>
          </div>

          <!-- Variants panel -->
          {#if expandedProduct === p.id}
            <div class="variant-panel">
              <div class="variant-header">
                <span style="font-size:0.8125rem;font-weight:600;color:var(--text-2);">Varian Produk</span>
                <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={() => openNewVariant(p.id)}>
                  <Plus size={13} /> Tambah Varian
                </button>
              </div>

              {#if loadingVariants[p.id]}
                <div style="padding:16px;"><div class="skel" style="height:40px;"></div></div>
              {:else if !productVariants[p.id]?.length}
                <div style="padding:16px 20px;color:var(--text-3);font-size:0.875rem;">Belum ada varian. Klik &ldquo;Tambah Varian&rdquo; untuk menambahkan.</div>
              {:else}
                <div class="variant-list">
                  {#each productVariants[p.id] as v (v.id)}
                    <div class="variant-row">
                      <div class="v-info">
                        <span style="font-weight:500;">{v.name}</span>
                        <span class="mono" style="font-size:0.75rem;color:var(--text-3);margin-left:8px;">{v.sku}</span>
                      </div>
                      <div style="display:flex;align-items:center;gap:10px;">
                        <div style="display:flex;flex-direction:column;align-items:flex-end;">
                          <span style="font-weight:600;color:var(--brand-600);">{currency(v.price)}</span>
                          <span style="font-size:0.75rem;color:var(--text-3);">HPP: {currency(v.investor_price)}</span>
                        </div>
                        {#if v.quota !== null && v.quota !== undefined}
                          <span style="font-size:0.8125rem;color:var(--text-2);">Kuota: {v.quota - v.quota_used}/{v.quota}</span>
                        {/if}
                        <Badge variant={v.status === 'active' ? 'success' : 'neutral'} size="sm">
                          {v.status === 'active' ? 'Aktif' : 'Nonaktif'}
                        </Badge>
                        <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEditVariant(p.id, v)} aria-label="Edit varian"><Edit2 size={13} /></button>
                        <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => deleteVariant(p.id, v.id, v.name)} aria-label="Hapus varian"><X size={13} /></button>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add/Edit Product Modal -->
<Modal bind:open={modalOpen} title={editTarget ? 'Edit Produk' : 'Tambah Produk'} size="lg">
  <div class="modal-body">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
      <div class="field">
        <label class="label" for="p-name">Nama Produk</label>
        <input id="p-name" class="input" bind:value={form.name} placeholder="Nama produk" />
      </div>
      <div class="field">
        <label class="label" for="p-sku">SKU</label>
        <input id="p-sku" class="input" bind:value={form.sku} placeholder="WH-001" />
      </div>
      <div class="field">
        <label class="label" for="p-cat">Kategori</label>
        <Select
          id="p-cat"
          bind:value={form.category_id}
          options={[{ value: '', label: '(Tanpa Kategori)' }, ...categories.map(c => ({ value: c.id, label: c.name }))]}
        />
      </div>
      <div class="field">
        <label class="label" for="p-price">Harga Jual (Rp)</label>
        <input id="p-price" class="input" type="number" min="0" bind:value={form.price} />
      </div>
      <div class="field">
        <label class="label" for="p-hpp">Harga Investor / HPP (Rp)</label>
        <input id="p-hpp" class="input" type="number" min="0" bind:value={form.investor_price} />
      </div>
      <div class="field">
        <label class="label" for="p-investor">Investor</label>
        <Select
          id="p-investor"
          bind:value={form.investor_id}
          options={[{ value: '', label: '(Tanpa Investor)' }, ...investors.map(i => ({ value: i.id, label: i.name }))]}
        />
      </div>
      <div class="field">
        <label class="label" for="p-quota">Kuota (kosong = tak terbatas)</label>
        <input id="p-quota" class="input" type="number" min="0" bind:value={form.quota} placeholder="Tidak terbatas" />
      </div>
      <div class="field">
        <label class="label" for="p-status">Status</label>
        <Select
          id="p-status"
          bind:value={form.status}
          options={[
            { value: 'active',   label: 'Aktif'    },
            { value: 'inactive', label: 'Nonaktif' },
            { value: 'draft',    label: 'Draft'    },
          ]}
        />
      </div>
    </div>
    <div class="toggle-group">
      <label class="toggle-item"><input type="checkbox" bind:checked={form.has_variants} /><span>Multi Varian</span></label>
      <label class="toggle-item"><input type="checkbox" bind:checked={form.ticket_required} /><span>Butuh Tiket</span></label>
      <label class="toggle-item"><input type="checkbox" bind:checked={form.print_enabled} /><span>Print Tiket</span></label>
      <label class="toggle-item"><input type="checkbox" bind:checked={form.scan_enabled} /><span>Scan Tiket</span></label>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (modalOpen=false)}>Batal</button>
    <button class="btn btn-primary" on:click={save} disabled={saving || !form.name || !form.sku}>
      {#if saving}<Spinner size={14} color="#fff" />{/if} {editTarget ? 'Simpan' : 'Tambah'}
    </button>
  </div>
</Modal>

<!-- Variant Modal -->
<Modal bind:open={variantModalOpen} title={editVariantTarget ? 'Edit Varian' : 'Tambah Varian'} size="sm">
  <div class="modal-body">
    <div style="display:flex;flex-direction:column;gap:14px;">
      <div class="field">
        <label class="label" for="v-name">Nama Varian</label>
        <input id="v-name" class="input" bind:value={variantForm.name} placeholder="Contoh: Dewasa, Anak-anak, S, M, L" />
      </div>
      <div class="field">
        <label class="label" for="v-sku">SKU Varian</label>
        <input id="v-sku" class="input" bind:value={variantForm.sku} placeholder="WH-001-D" />
      </div>
      <div class="field">
        <label class="label" for="v-price">Harga Jual (Rp)</label>
        <input id="v-price" class="input" type="number" min="0" bind:value={variantForm.price} />
      </div>
      <div class="field">
        <label class="label" for="v-hpp">Harga Investor / HPP (Rp)</label>
        <input id="v-hpp" class="input" type="number" min="0" bind:value={variantForm.investor_price} />
      </div>
      <div class="field">
        <label class="label" for="v-quota">Kuota (kosong = tak terbatas)</label>
        <input id="v-quota" class="input" type="number" min="0" bind:value={variantForm.quota} placeholder="Tidak terbatas" />
      </div>
      <div class="field">
        <label class="label" for="v-status">Status</label>
        <Select
          id="v-status"
          bind:value={variantForm.status}
          options={[
            { value: 'active',   label: 'Aktif'    },
            { value: 'inactive', label: 'Nonaktif' },
          ]}
        />
      </div>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (variantModalOpen=false)}>Batal</button>
    <button class="btn btn-primary" on:click={saveVariant} disabled={saving || !variantForm.name || !variantForm.sku}>
      {#if saving}<Spinner size={14} color="#fff" />{/if} {editVariantTarget ? 'Simpan' : 'Tambah'}
    </button>
  </div>
</Modal>

<!-- Delete Product Modal -->
<Modal bind:open={deleteOpen} title="Hapus Produk" size="sm">
  <div class="modal-body"><p>Yakin ingin menghapus <strong>{deleteTarget?.name}</strong>? Tindakan ini tidak dapat dibatalkan.</p></div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (deleteOpen=false)}>Batal</button>
    <button class="btn btn-danger" on:click={deleteProduct} disabled={saving}>{#if saving}<Spinner size={14} color="#fff" />{/if} Hapus</button>
  </div>
</Modal>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1100px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .filters { display:flex;gap:10px;flex-wrap:wrap; }
  .search-wrap { display:flex;align-items:center;gap:8px;padding:0 12px;height:36px;background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-md);flex:1;min-width:200px; }
  .search-input { flex:1;border:none;outline:none;background:transparent;font-family:var(--font-sans);font-size:0.875rem;color:var(--text-1); }
  .search-input::placeholder { color:var(--text-3); }
  .products-list { display:flex;flex-direction:column;gap:8px; }
  .product-row { overflow:hidden; }
  .product-main { display:flex;align-items:center;gap:12px;padding:12px 16px;flex-wrap:wrap; }
  .p-icon { width:32px;height:32px;border-radius:var(--r-md);background:var(--brand-50);display:flex;align-items:center;justify-content:center;flex-shrink:0; }
  [data-theme='dark'] .p-icon { background:rgba(99,102,241,0.12); }
  .p-info { flex:1;min-width:160px; }
  /* Variants */
  .variant-panel { border-top:1px solid var(--border);background:var(--bg-subtle); }
  .variant-header { display:flex;align-items:center;justify-content:space-between;padding:10px 16px;border-bottom:1px solid var(--border); }
  .variant-list { display:flex;flex-direction:column; }
  .variant-row { display:flex;align-items:center;justify-content:space-between;padding:10px 16px;border-bottom:1px solid var(--border-subtle);flex-wrap:wrap;gap:8px; }
  .variant-row:last-child { border-bottom:none; }
  .v-info { display:flex;align-items:center;flex:1;min-width:120px; }
  /* Form */
  .toggle-group { display:flex;flex-wrap:wrap;gap:12px;padding:16px;background:var(--bg-subtle);border-radius:var(--r-md);margin-top:8px; }
  .toggle-item { display:flex;align-items:center;gap:6px;font-size:0.875rem;color:var(--text-2);cursor:pointer;user-select:none; }
  .toggle-item input { accent-color:var(--brand-500);cursor:pointer; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
</style>
