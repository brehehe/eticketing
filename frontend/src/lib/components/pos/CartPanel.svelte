<script lang="ts">
  import { cart, cartSummary } from '$lib/stores/cart';
  import { currency } from '$lib/utils/format';
  import { Plus, Minus, Trash2, X, Tag, CreditCard } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let discount = 0;
  export let discountType: 'nominal' | 'persen' = 'nominal';
  export let voucherCode = '';
  export let voucherApplied = false;

  const dispatch = createEventDispatcher<{ checkout: void; voucherApply: string }>();

  $: items = $cart;
  $: subtotal = $cartSummary.subtotal;
  $: qty = $cartSummary.qty;
  $: discountAmt = discountType === 'persen'
    ? Math.round(subtotal * discount / 100)
    : discount;
  $: total = Math.max(0, subtotal - discountAmt);
</script>

<div class="cart-panel">
  <div class="cart-head">
    <h3>Keranjang
      {#if qty > 0}
        <span class="cart-badge">{qty}</span>
      {/if}
    </h3>
    {#if items.length > 0}
      <button class="btn btn-ghost btn-sm danger" on:click={cart.clear} aria-label="Kosongkan keranjang">
        <Trash2 size={13} /> Kosongkan
      </button>
    {/if}
  </div>

  <div class="cart-items" role="list" aria-label="Item keranjang">
    {#if items.length === 0}
      <div class="cart-empty" role="listitem">
        <svg width="44" height="44" viewBox="0 0 44 44" fill="none" aria-hidden="true">
          <rect width="44" height="44" rx="14" fill="var(--bg-muted)"/>
          <path d="M12 12h4l3 14h12l2-9H16" stroke="var(--text-3)" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="20" cy="30" r="1.5" fill="var(--text-3)"/>
          <circle cx="29" cy="30" r="1.5" fill="var(--text-3)"/>
        </svg>
        <p>Keranjang kosong</p>
        <span>Pilih produk dari daftar</span>
      </div>
    {:else}
      {#each items as item (item.product.id + (item.variant?.id ?? ''))}
        <div class="cart-item" role="listitem">
          <div class="ci-top">
            <span class="ci-name">
              {item.product.name}{item.variant ? ` — ${item.variant.name}` : ''}
            </span>
            <button class="ci-remove" on:click={() => cart.removeItem(item.product.id, item.variant?.id)} aria-label="Hapus {item.product.name}">
              <X size={12} />
            </button>
          </div>
          <div class="ci-bot">
            <span class="ci-unit">{currency(item.unit_price)}</span>
            <div class="qty-ctrl">
              <button class="qty-btn" on:click={() => cart.updateQty(item.product.id, item.qty - 1, item.variant?.id)} aria-label="Kurangi">
                <Minus size={11} />
              </button>
              <span class="qty-val">{item.qty}</span>
              <button class="qty-btn" on:click={() => cart.updateQty(item.product.id, item.qty + 1, item.variant?.id)} aria-label="Tambah">
                <Plus size={11} />
              </button>
            </div>
            <span class="ci-sub">{currency(item.subtotal)}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <div class="cart-foot">
    <div class="disc-row">
      <Tag size={13} style="color:var(--text-3);flex-shrink:0;" />
      <input
        class="disc-input"
        type="number" min="0"
        bind:value={discount}
        placeholder="Diskon"
        aria-label="Nominal diskon"
      />
      <button
        class="disc-type"
        on:click={() => (discountType = discountType === 'nominal' ? 'persen' : 'nominal')}
        title="Toggle tipe diskon"
        aria-label="Toggle persen/nominal"
      >{discountType === 'persen' ? '%' : 'Rp'}</button>
    </div>

    <div class="vou-row">
      <input
        class="input input-sm"
        style="flex:1;min-width:0;"
        type="text"
        bind:value={voucherCode}
        placeholder="Kode voucher"
        aria-label="Kode voucher"
        disabled={voucherApplied}
      />
      <button
        class="btn btn-secondary btn-sm"
        disabled={!voucherCode || voucherApplied}
        on:click={() => dispatch('voucherApply', voucherCode)}
      >{voucherApplied ? '\u2713 Aktif' : 'Pakai'}</button>
    </div>

    <div class="divider"></div>

    <div class="sum-row"><span>Subtotal</span><span>{currency(subtotal)}</span></div>
    {#if discountAmt > 0}
      <div class="sum-row" style="color:var(--rose);"><span>Diskon</span><span>- {currency(discountAmt)}</span></div>
    {/if}
    <div class="sum-row total"><span>Total</span><span>{currency(total)}</span></div>

    <button
      class="btn btn-primary btn-lg checkout-btn"
      disabled={items.length === 0}
      on:click={() => dispatch('checkout')}
    >
      <CreditCard size={17} />
      Bayar {items.length > 0 ? currency(total) : ''}
    </button>
  </div>
</div>

<style>
  .cart-panel {
    background: var(--bg-surface);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .cart-head {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }
  .cart-head h3 { font-size: 0.9375rem; display: flex; align-items: center; gap: 8px; }
  .cart-badge {
    display: inline-flex; align-items: center; justify-content: center;
    min-width: 20px; height: 20px; padding: 0 5px;
    background: var(--brand-500); color: #fff;
    font-size: 0.6875rem; font-weight: 700;
    border-radius: var(--r-full);
  }
  .btn.danger:hover { color: var(--rose); }

  .cart-items { flex: 1; overflow-y: auto; padding: 8px; display: flex; flex-direction: column; gap: 4px; }

  .cart-empty {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 8px; padding: 32px 16px;
    color: var(--text-3); text-align: center;
  }
  .cart-empty p { font-size: 0.875rem; font-weight: 500; color: var(--text-2); }
  .cart-empty span { font-size: 0.8125rem; }

  .cart-item {
    background: var(--bg-subtle); border-radius: var(--r-md);
    padding: 8px 10px; display: flex; flex-direction: column; gap: 6px;
  }
  .ci-top { display: flex; align-items: flex-start; justify-content: space-between; gap: 6px; }
  .ci-name { font-size: 0.8125rem; font-weight: 500; color: var(--text-1); line-height: 1.4; flex: 1; }
  .ci-remove {
    flex-shrink: 0; width: 18px; height: 18px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--r-sm); color: var(--text-3);
    cursor: pointer; border: none; background: none;
    transition: color var(--ease-fast), background var(--ease-fast);
  }
  .ci-remove:hover { color: var(--rose); background: rgba(244,63,94,0.08); }
  .ci-bot { display: flex; align-items: center; gap: 8px; }
  .ci-unit { font-size: 0.75rem; color: var(--text-3); flex: 1; }
  .qty-ctrl { display: flex; align-items: center; gap: 4px; }
  .qty-btn {
    width: 22px; height: 22px;
    border-radius: var(--r-sm);
    border: 1px solid var(--border); background: var(--bg-surface);
    display: flex; align-items: center; justify-content: center;
    cursor: pointer; color: var(--text-2);
    transition: background var(--ease-fast);
  }
  .qty-btn:hover { background: var(--bg-muted); color: var(--text-1); }
  .qty-val { font-size: 0.8125rem; font-weight: 600; min-width: 20px; text-align: center; }
  .ci-sub { font-size: 0.875rem; font-weight: 700; color: var(--text-1); }

  /* Footer */
  .cart-foot { padding: 12px 14px; border-top: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px; flex-shrink: 0; }

  .disc-row { display: flex; align-items: center; gap: 6px; }
  .disc-input {
    flex: 1; height: 32px; padding: 0 8px;
    font-size: 0.8125rem; font-family: var(--font-sans);
    background: var(--bg-subtle); border: 1px solid var(--border);
    border-radius: var(--r-sm); color: var(--text-1); outline: none;
    transition: border-color var(--ease-fast);
  }
  .disc-input:focus { border-color: var(--brand-400); }
  .disc-type {
    height: 32px; padding: 0 10px;
    font-size: 0.8125rem; font-weight: 600;
    background: var(--bg-muted); border: 1px solid var(--border);
    border-radius: var(--r-sm); cursor: pointer; color: var(--text-1);
    font-family: var(--font-sans); transition: background var(--ease-fast);
  }
  .disc-type:hover { background: var(--border); }

  .vou-row { display: flex; gap: 6px; }

  .sum-row {
    display: flex; justify-content: space-between;
    font-size: 0.875rem; color: var(--text-2);
  }
  .sum-row.total {
    font-size: 1rem; font-weight: 700;
    color: var(--text-1); margin-top: 2px;
  }

  .checkout-btn { width: 100%; margin-top: 4px; }
</style>
