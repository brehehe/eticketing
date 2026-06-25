<script lang="ts">
  import { currency, datetime } from '$lib/utils/format';
  import type { Transaction, VenueSettings } from '$lib/types';

  export let transaction: Transaction;
  export let venue: VenueSettings;

  $: items = transaction.items ?? [];
  $: payments = transaction.payments ?? [];
</script>

<!-- 80mm receipt preview -->
<div class="receipt" role="document" aria-label="Preview struk">
  <!-- Header -->
  <div class="receipt-header">
    {#if venue.logo}
      <img src={venue.logo} alt={venue.name} class="receipt-logo" />
    {/if}
    <h2 class="receipt-venue">{venue.name}</h2>
    {#if venue.address}
      <p class="receipt-address">{venue.address}</p>
    {/if}
    {#if venue.phone}
      <p class="receipt-address">{venue.phone}</p>
    {/if}
  </div>

  <div class="receipt-divider">{'='.repeat(32)}</div>

  <!-- Invoice info -->
  <div class="receipt-info">
    <div class="info-row">
      <span>Invoice</span>
      <span class="mono">{transaction.invoice}</span>
    </div>
    <div class="info-row">
      <span>Tanggal</span>
      <span>{datetime(transaction.created_at)}</span>
    </div>
  </div>

  <div class="receipt-divider">{'-'.repeat(32)}</div>

  <!-- Items -->
  <div class="receipt-items">
    {#each items as item}
      <div class="item-row">
        <span class="item-name">
          {item.product?.name ?? 'Item'}
          {item.variant ? ` (${item.variant.name})` : ''}
        </span>
        <span class="item-price">{currency(item.unit_price)}</span>
      </div>
      <div class="item-sub">
        <span>x{item.qty}</span>
        <span>{currency(item.subtotal)}</span>
      </div>
    {/each}
  </div>

  <div class="receipt-divider">{'-'.repeat(32)}</div>

  <!-- Summary -->
  <div class="receipt-summary">
    <div class="sum-row">
      <span>Subtotal</span>
      <span>{currency(transaction.subtotal)}</span>
    </div>
    {#if transaction.discount > 0}
      <div class="sum-row">
        <span>Diskon</span>
        <span>-{currency(transaction.discount)}</span>
      </div>
    {/if}
    <div class="sum-row total">
      <span>TOTAL</span>
      <span>{currency(transaction.total)}</span>
    </div>
    {#each payments as p}
      <div class="sum-row">
        <span>{p.method.toUpperCase()}</span>
        <span>{currency(p.amount)}</span>
      </div>
    {/each}
    {#if transaction.change > 0}
      <div class="sum-row">
        <span>Kembalian</span>
        <span>{currency(transaction.change)}</span>
      </div>
    {/if}
  </div>

  <div class="receipt-divider">{'='.repeat(32)}</div>

  <div class="receipt-footer">
    <p>Terima kasih!</p>
    <p>{venue.phone}</p>
  </div>
</div>

<style>
  .receipt {
    width: 302px; /* ~80mm at 96dpi */
    background: #fff;
    color: #000;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.5;
    padding: 12px;
    border: 1px dashed #ccc;
    border-radius: 4px;
  }
  .receipt-header { text-align:center; margin-bottom:8px; }
  .receipt-logo   { max-width:80px; margin:0 auto 6px; }
  .receipt-venue  { font-size:14px; font-weight:bold; margin-bottom:2px; }
  .receipt-address{ font-size:11px; color:#444; }
  .receipt-divider{ text-align:center; color:#666; margin:6px 0; font-size:11px; }
  .receipt-info   { margin:4px 0; }
  .info-row {
    display:flex; justify-content:space-between;
    font-size:11px; margin-bottom:2px;
  }
  .receipt-items  { margin:4px 0; }
  .item-row {
    display:flex; justify-content:space-between;
    font-size:11px; font-weight:bold;
  }
  .item-name  { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .item-price { flex-shrink:0; margin-left:8px; }
  .item-sub {
    display:flex; justify-content:space-between;
    font-size:10px; color:#555; padding-left:8px; margin-bottom:4px;
  }
  .receipt-summary { margin:4px 0; }
  .sum-row {
    display:flex; justify-content:space-between;
    font-size:11px; margin-bottom:2px;
  }
  .sum-row.total {
    font-size:13px; font-weight:bold;
    border-top:1px solid #000; padding-top:4px; margin-top:4px;
  }
  .receipt-footer { text-align:center; margin-top:8px; font-size:11px; color:#555; }
</style>
