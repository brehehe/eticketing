<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { cart } from '$lib/stores/cart';
  import { currency } from '$lib/utils/format';
  import { api } from '$lib/api/client';
  import Modal from '$components/ui/Modal.svelte';
  import Spinner from '$components/ui/Spinner.svelte';
  import { Check, QrCode, Banknote, CreditCard, Smartphone, RefreshCw } from 'lucide-svelte';

  export let open = false;
  export let discountAmt = 0;
  export let total = 0;
  export let loading = false;

  const dispatch = createEventDispatcher<{ confirm: { method: string; cashAmt: number }; close: void }>();

  interface PMItem {
    id: string;
    label: string;
    icon: any;
    method_id: string; // API method name for transaction
  }

  // Default fallback list
  const defaultPM: PMItem[] = [
    { id: 'cash',     label: 'Tunai',    icon: Banknote,   method_id: 'cash'     },
    { id: 'qris',     label: 'QRIS',     icon: QrCode,     method_id: 'qris'     },
    { id: 'transfer', label: 'Transfer', icon: CreditCard, method_id: 'transfer' },
    { id: 'debit',    label: 'Debit',    icon: CreditCard, method_id: 'debit'    },
    { id: 'ewallet',  label: 'E-Wallet', icon: Smartphone, method_id: 'ewallet'  },
  ];

  let pmList: PMItem[] = defaultPM;
  let loadingPM = false;

  async function loadPaymentMethods() {
    loadingPM = true;
    try {
      const res = await api.get<any[]>('/payment-methods');
      const d = (res as any)?.data;
      if (Array.isArray(d) && d.length > 0) {
        // Map API data to icon list
        const iconMap: Record<string, any> = {
          'tunai': Banknote, 'cash': Banknote,
          'qris': QrCode,
          'transfer': CreditCard, 'transfer bank': CreditCard,
          'debit': CreditCard,
          'e-wallet': Smartphone, 'ewallet': Smartphone, 'gopay': Smartphone,
        };
        pmList = d
          .filter((m: any) => m.status === 'active')
          .map((m: any) => ({
            id: m.id,
            label: m.name,
            icon: iconMap[m.name.toLowerCase()] ?? CreditCard,
            method_id: m.name.toLowerCase().replace(/[^a-z0-9]/g, '_'),
          }));
      }
    } catch {
      pmList = defaultPM;
    } finally {
      loadingPM = false;
    }
  }

  $: if (open && pmList === defaultPM) loadPaymentMethods();

  let paymentMethod = 'cash';
  let cashInput = '';

  $: items = $cart;
  $: cashAmt = parseInt(cashInput.replace(/[^0-9]/g, '')) || 0;
  $: change  = Math.max(0, cashAmt - total);
  $: selectedPM = pmList.find(p => p.id === paymentMethod);
  $: isCash = selectedPM?.label.toLowerCase().includes('tunai') || selectedPM?.label.toLowerCase() === 'cash' || paymentMethod === 'cash';
  $: isQris = selectedPM?.label.toLowerCase().includes('qris');
  $: canPay = isCash ? cashAmt >= total && total > 0 : total > 0;

  // Reset when modal opens
  $: if (open) { paymentMethod = pmList[0]?.id ?? 'cash'; cashInput = ''; }

  function formatCash(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9]/g, '');
    cashInput = raw ? Number(raw).toLocaleString('id-ID') : '';
  }

  function quickAmounts(): number[] {
    return [...new Set([
      total,
      Math.ceil(total / 10000) * 10000,
      Math.ceil(total / 50000) * 50000,
      Math.ceil(total / 100000) * 100000,
    ])].slice(0, 4);
  }
</script>

<Modal bind:open title="Pembayaran" size="md" on:close={() => dispatch('close')}>
  <div class="modal-body">
    <!-- Order summary -->
    <div class="co-summary">
      {#each items as item}
        <div class="co-row">
          <span class="co-item-name">{item.product.name}{item.variant ? ` \u2014 ${item.variant.name}` : ''} &times;{item.qty}</span>
          <span>{currency(item.subtotal)}</span>
        </div>
      {/each}
      {#if discountAmt > 0}
        <div class="co-row" style="color:var(--rose);"><span>Diskon</span><span>- {currency(discountAmt)}</span></div>
      {/if}
      <div class="co-total"><span>Total Bayar</span><span>{currency(total)}</span></div>
    </div>

    <!-- Payment method -->
    <div style="display:flex;align-items:center;justify-content:space-between;margin:16px 0 10px;">
      <p class="label" style="margin:0;">Metode Pembayaran</p>
      {#if loadingPM}
        <Spinner size={14} />
      {:else}
        <button class="btn btn-ghost btn-sm btn-icon" on:click={loadPaymentMethods} title="Refresh metode">
          <RefreshCw size={13} />
        </button>
      {/if}
    </div>

    {#if loadingPM}
      <div class="skel" style="height:56px;border-radius:var(--r-md);"></div>
    {:else}
      <div class="pm-grid">
        {#each pmList as pm}
          <button
            class="pm-btn"
            class:active={paymentMethod === pm.id}
            on:click={() => { paymentMethod = pm.id; cashInput = ''; }}
          >
            <svelte:component this={pm.icon} size={16} />
            <span>{pm.label}</span>
          </button>
        {/each}
      </div>
    {/if}

    <!-- Cash input -->
    {#if isCash}
      <div class="field" style="margin-top:16px;">
        <label class="label" for="cash-amt">Uang Diterima</label>
        <input
          id="cash-amt"
          class="input input-lg"
          style="font-size:1.375rem;font-weight:700;font-family:var(--font-mono);letter-spacing:0.02em;"
          type="text" inputmode="numeric"
          value={cashInput}
          on:input={formatCash}
          placeholder="0"
          aria-label="Nominal uang yang diterima"
        />
      </div>
      <div style="display:flex;gap:6px;margin-top:8px;flex-wrap:wrap;">
        {#each quickAmounts() as amt}
          <button class="btn btn-secondary btn-sm" on:click={() => (cashInput = amt.toLocaleString('id-ID'))}>
            {currency(amt)}
          </button>
        {/each}
      </div>
      {#if change > 0}
        <div class="change-box">
          <span>Kembalian</span>
          <span class="change-val">{currency(change)}</span>
        </div>
      {/if}
    {/if}

    {#if isQris}
      <div class="qris-wrap">
        <div class="qris-inner"><QrCode size={96} style="color:var(--text-2);" /></div>
        <p style="color:var(--text-2);font-size:0.8125rem;text-align:center;margin-top:10px;">
          QR dari payment gateway akan muncul di sini.<br/>Menunggu konfirmasi otomatis&hellip;
        </p>
      </div>
    {/if}
  </div>

  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => dispatch('close')} disabled={loading}>Batal</button>
    <button
      class="btn btn-primary btn-lg"
      disabled={!canPay || loading}
      on:click={() => dispatch('confirm', { method: selectedPM?.method_id ?? paymentMethod, cashAmt })}
    >
      {#if loading}<Spinner size={15} color="#fff" />{/if}
      <Check size={15} />
      Konfirmasi Bayar
    </button>
  </div>
</Modal>

<style>
  .co-summary {
    background: var(--bg-subtle);
    border-radius: var(--r-md);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .co-row { display:flex; justify-content:space-between; font-size:0.875rem; color:var(--text-2); gap:12px; }
  .co-item-name { flex:1; }
  .co-total {
    display:flex; justify-content:space-between;
    font-size:1rem; font-weight:700; color:var(--text-1);
    padding-top:8px; border-top:1px solid var(--border); margin-top:4px;
  }
  .pm-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 6px;
  }
  .pm-btn {
    display:flex; flex-direction:column; align-items:center; gap:5px;
    padding:10px 6px;
    border-radius:var(--r-md);
    border:1px solid var(--border);
    background:var(--bg-surface);
    color:var(--text-2);
    font-size:0.6875rem; font-weight:500;
    cursor:pointer; font-family:var(--font-sans);
    transition:all var(--ease-fast);
    white-space:nowrap; overflow:hidden; text-overflow:ellipsis;
  }
  .pm-btn:hover   { border-color:var(--brand-300); color:var(--text-1); }
  .pm-btn.active  {
    border-color:var(--brand-500);
    background:var(--brand-50);
    color:var(--brand-600);
    box-shadow:var(--ring-brand);
  }
  [data-theme='dark'] .pm-btn.active { background:rgba(99,102,241,0.14); color:var(--brand-300); }
  .change-box {
    display:flex; justify-content:space-between; align-items:center;
    margin-top:12px; padding:10px 14px;
    background:rgba(16,185,129,0.08); border:1px solid rgba(16,185,129,0.2);
    border-radius:var(--r-md); font-size:0.875rem; color:var(--text-2);
  }
  .change-val { font-size:1.125rem; font-weight:700; color:var(--emerald); }
  .qris-wrap { display:flex; flex-direction:column; align-items:center; padding:20px; }
  .qris-inner { padding:20px; background:var(--bg-subtle); border-radius:var(--r-lg); border:1px solid var(--border); }
</style>
