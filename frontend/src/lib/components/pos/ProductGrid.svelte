<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { currency } from '$lib/utils/format';
  import type { Product } from '$lib/types';
  import Badge from '$components/ui/Badge.svelte';
  import { Plus } from 'lucide-svelte';

  export let products: Product[] = [];
  export let searchQuery = '';
  export let selectedCategory = 'all';

  const dispatch = createEventDispatcher<{ add: Product }>();

  const categoryEmoji: Record<string, string> = {
    wahana: '\uD83C\uDFA1', makanan: '\uD83C\uDF7D\uFE0F',
    minuman: '\uD83E\uDD64', souvenir: '\uD83D\uDECD\uFE0F',
  };

  $: filtered = products.filter(p => {
    const matchCat = selectedCategory === 'all' || p.category_id === selectedCategory;
    const q = searchQuery.toLowerCase();
    return matchCat && p.status === 'active' &&
      (p.name.toLowerCase().includes(q) || p.sku.toLowerCase().includes(q));
  });
</script>

<div class="product-grid" role="list" aria-label="Daftar produk">
  {#each filtered as p (p.id)}
    <button
      class="product-card"
      on:click={() => dispatch('add', p)}
      role="listitem"
      aria-label="Tambah {p.name} ke keranjang"
    >
      <div class="p-emoji" aria-hidden="true">
        {categoryEmoji[p.category_id] ?? '\uD83D\uDCE6'}
      </div>
      <div class="p-info">
        <span class="p-name">{p.name}</span>
        <span class="p-sku">{p.sku}</span>
        <div class="p-row">
          <span class="p-price">{currency(p.price)}</span>
          {#if p.has_variants}
            <Badge variant="info" size="sm">Varian</Badge>
          {/if}
          {#if p.ticket_required}
            <Badge variant="brand" size="sm">Tiket</Badge>
          {/if}
        </div>
        {#if p.quota !== undefined}
          <div class="quota-bar">
            <div class="quota-fill" style="width:{Math.min(100, (p.quota_used / p.quota) * 100)}%"></div>
          </div>
          <span class="quota-lbl">{(p.quota ?? 0) - p.quota_used} sisa kuota</span>
        {/if}
      </div>
      <div class="p-add" aria-hidden="true"><Plus size={18} /></div>
    </button>
  {:else}
    <p style="grid-column:1/-1;color:var(--text-3);padding:32px;text-align:center;">Tidak ada produk</p>
  {/each}
</div>

<style>
  .product-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(155px, 1fr));
    gap: 12px;
    padding: 16px;
  }
  .product-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 14px 12px 12px;
    cursor: pointer;
    text-align: left;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-family: var(--font-sans);
    width: 100%;
    transition: transform var(--ease-fast), box-shadow var(--ease-fast), border-color var(--ease-fast);
  }
  .product-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
    border-color: var(--brand-300);
  }
  .product-card:hover .p-add { opacity: 1; transform: scale(1); }
  .p-emoji { font-size: 1.75rem; line-height: 1; }
  .p-info { display: flex; flex-direction: column; gap: 3px; }
  .p-name { font-size: 0.875rem; font-weight: 600; color: var(--text-1); line-height: 1.35; }
  .p-sku  { font-size: 0.6875rem; color: var(--text-3); font-family: var(--font-mono); }
  .p-row  { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; margin-top: 4px; }
  .p-price { font-size: 0.9375rem; font-weight: 700; color: var(--brand-600); }
  .quota-bar {
    height: 3px; background: var(--bg-muted);
    border-radius: var(--r-full); overflow: hidden; margin-top: 6px;
  }
  .quota-fill { height: 100%; background: var(--brand-500); border-radius: var(--r-full); }
  .quota-lbl  { font-size: 0.6875rem; color: var(--text-3); }
  .p-add {
    position: absolute; top: 10px; right: 10px;
    width: 28px; height: 28px;
    background: var(--brand-500); color: #fff;
    border-radius: var(--r-full);
    display: flex; align-items: center; justify-content: center;
    opacity: 0; transform: scale(0.8);
    transition: opacity var(--ease-fast), transform var(--ease-spring);
  }
</style>
