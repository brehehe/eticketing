<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { cart, cartSummary } from '$lib/stores/cart';
  import { toast } from '$lib/stores/toast';
  import { currency, relativeTime, datetime } from '$lib/utils/format';
  import { productsApi, transactionsApi, categoriesApi, variantsApi, settingsApi } from '$lib/api/resources';
  import { drafts } from '$lib/stores/drafts';
  import type { Draft } from '$lib/stores/drafts';
  import type { Product, ProductVariant } from '$lib/types';
  import ProductGrid from '$lib/components/pos/ProductGrid.svelte';
  import CartPanel from '$lib/components/pos/CartPanel.svelte';
  import CheckoutModal from '$lib/components/pos/CheckoutModal.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { Search, Printer, RefreshCw, History, BookmarkPlus, Bookmark, Trash2, RotateCcw, X, Clock, ShoppingCart, Bluetooth, BluetoothOff, Usb, Server } from 'lucide-svelte';
  import PrintModal from '$lib/components/print/PrintModal.svelte';
  import { bluetooth, btState, btDevice } from '$lib/stores/bluetooth';
  import { serial, serialState, serialPortName } from '$lib/stores/serial';
  import { bridge, bridgeState, bridgePortName } from '$lib/stores/bridge';

  let printModalOpen = false;
  let printTxData: any = null;
  let printItems: any[] = [];
  let printPayments: any[] = [];
  let printTickets: any[] = [];
  let historyTickets: any[] = [];

  let products: Product[] = [];
  let dbCategories: { id: string; name: string; slug: string }[] = [];
  let loadingProducts = true;
  let searchQuery = '';
  let selectedCategory = 'all';
  let discount = 0;
  let discountType: 'nominal' | 'persen' = 'nominal';
  let voucherCode = '';
  let voucherApplied = false;
  let checkoutOpen = false;
  let variantModalOpen = false;
  let successOpen = false;
  let selectedProduct: Product | null = null;
  let selectedVariants: any[] = [];
  let loadingVariants = false;
  let loading = false;
  let lastInvoice = '';
  let lastTxId = '';

  let refundOpen = false;
  let refundTarget: { id: string; invoice: string } | null = null;
  let refunding = false;

  type Panel = 'none' | 'history' | 'drafts';
  let activePanel: Panel = 'none';
  let txHistory: any[] = [];
  let loadingHistory = false;
  let historyDetail: any = null;
  let historyDetailOpen = false;
  let saveDraftOpen = false;
  let draftName = '';

  let activeMobileTab: 'products' | 'cart' | 'history' | 'drafts' = 'products';

  function setMobileTab(tab: 'products' | 'cart' | 'history' | 'drafts') {
    activeMobileTab = tab;
    if (tab === 'history') {
      activePanel = 'history';
      loadHistory();
    } else if (tab === 'drafts') {
      activePanel = 'drafts';
    }
  }

  $: categories = [{ id: 'all', name: 'Semua' }, ...dbCategories];
  $: filteredByCategory = selectedCategory === 'all' ? products : products.filter(p => p.category_id === selectedCategory);
  $: subtotal    = $cartSummary.subtotal;
  $: discountAmt = discountType === 'persen' ? Math.round(subtotal * discount / 100) : discount;
  $: total       = Math.max(0, subtotal - discountAmt);
  $: allDrafts   = $drafts;

  let venueSettings = { name: 'TiketKu', address: '', phone: '', email: '' };
  let venueFooter1 = 'Terima kasih!';
  let venueFooter2 = '';
  let paperWidth = '80mm';
  let qrEnabled = true;

  async function loadAll() {
    loadingProducts = true;
    try {
      const [prodRes, catRes, setRes] = await Promise.all([
        productsApi.list({ status: 'active', per_page: '200' }),
        categoriesApi.list(),
        settingsApi.get().catch(() => null),
      ]);
      const d = (prodRes as any)?.data;
      products = Array.isArray(d) ? d : [];
      const cd = (catRes as any)?.data;
      dbCategories = Array.isArray(cd) ? cd : [];

      const sd = (setRes as any)?.data;
      if (sd) {
        if (sd.venue) {
          venueSettings = {
            name: sd.venue.name || 'TiketKu',
            address: sd.venue.address || '',
            phone: sd.venue.phone || '',
            email: sd.venue.email || '',
          };
          venueFooter1 = sd.venue.footer_line_1 ?? 'Terima kasih!';
          venueFooter2 = sd.venue.footer_line_2 ?? '';
        }
        if (sd.printer?.paper_width) {
          paperWidth = sd.printer.paper_width;
        }
        if (sd.ticket) {
          qrEnabled = sd.ticket.qr_enabled !== false;
        }
      }
    } catch (e: any) {
      toast.error('Gagal memuat data', e.message);
    } finally { loadingProducts = false; }
  }

  async function loadHistory() {
    loadingHistory = true;
    try {
      const res = await transactionsApi.list({ per_page: '20' });
      const d = (res as any)?.data;
      txHistory = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat riwayat', e.message);
    } finally { loadingHistory = false; }
  }

  async function openHistoryDetail(tx: any) {
    try {
      historyTickets = [];
      const [res, tRes] = await Promise.all([
        transactionsApi.get(tx.id),
        transactionsApi.reprint(tx.id).catch(() => ({ data: [] }))
      ]);
      historyDetail = (res as any)?.data;
      historyTickets = (tRes as any)?.data ?? [];
      historyDetailOpen = true;
    } catch (e: any) { toast.error('Gagal memuat detail', e.message); }
  }

  async function openPrintModal(txId: string) {
    try {
      const [txRes, ticketRes] = await Promise.all([
        transactionsApi.get(txId),
        transactionsApi.reprint(txId),
      ]);
      const txData = (txRes as any)?.data;
      const ticketData = (ticketRes as any)?.data ?? [];
      printTxData  = txData?.transaction ?? null;
      printItems   = txData?.items   ?? [];
      printPayments= txData?.payments ?? [];
      printTickets = ticketData;
      printModalOpen = true;
    } catch (e: any) { toast.error('Gagal memuat data cetak', e.message); }
  }

  function confirmRefund(txId: string, invoice: string) {
    refundTarget = { id: txId, invoice };
    refundOpen = true;
  }

  async function executeRefund() {
    if (!refundTarget) return;
    refunding = true;
    try {
      await transactionsApi.refund(refundTarget.id);
      toast.success('Refund berhasil', refundTarget.invoice);
      refundOpen = false;
      historyDetailOpen = false;
      await loadHistory();
    } catch (e: any) {
      toast.error('Gagal refund', e.message);
    } finally {
      refunding = false;
    }
  }

  onMount(loadAll);

  function togglePanel(panel: Panel) {
    activePanel = activePanel === panel ? 'none' : panel;
    if (activePanel === 'history') loadHistory();
  }

  async function handleAdd(e: CustomEvent<Product>) {
    const p = e.detail;
    if (p.has_variants) {
      selectedProduct = p;
      selectedVariants = [];
      loadingVariants = true;
      variantModalOpen = true;
      try {
        const res = await variantsApi.list(p.id);
        const d = (res as any)?.data;
        selectedVariants = Array.isArray(d) ? d.filter((v: any) => v.status === 'active') : [];
      } catch {
        selectedVariants = (p.variants ?? []) as any[];
      } finally { loadingVariants = false; }
    } else {
      cart.addItem(p);
      toast.success('Ditambahkan', p.name);
    }
  }

  function addVariant(v: any) {
    if (!selectedProduct) return;
    cart.addItem(selectedProduct, v as ProductVariant);
    variantModalOpen = false;
    selectedVariants = [];
    toast.success('Ditambahkan', `${selectedProduct.name} \u2014 ${v.name}`);
  }

  function closeVariantModal() { variantModalOpen = false; selectedVariants = []; }
  function closeCheckout()     { checkoutOpen = false; }

  async function handleConfirm(e: CustomEvent<{ method: string; cashAmt: number }>) {
    loading = true;
    try {
      const items = $cart.map(item => ({ product_id: item.product.id, variant_id: item.variant?.id ?? null, qty: item.qty, discount: item.discount }));
      const payments = [{ method: e.detail.method, amount: e.detail.cashAmt || total }];
      const res = await transactionsApi.create({ items, payments, discount: discountAmt, voucher_code: voucherApplied ? voucherCode : null });
      const d = (res as any)?.data;
      lastInvoice = d?.invoice ?? 'TK-XXXX';
      lastTxId    = d?.transaction?.id ?? '';
      cart.clear();
      checkoutOpen = false;
      successOpen = true;
      activeMobileTab = 'products';
      discount = 0; voucherCode = ''; voucherApplied = false;
      if (activePanel === 'history') loadHistory();
    } catch (err: any) {
      toast.error('Checkout gagal', err.message);
    } finally { loading = false; }
  }

  function openSaveDraft() {
    if ($cart.length === 0) { toast.warning('Keranjang kosong', ''); return; }
    draftName = `Draft ${new Date().toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' })}`;
    saveDraftOpen = true;
  }

  function saveDraftConfirm() {
    drafts.save($cart, { name: draftName, discount, discountType, voucherCode });
    cart.clear();
    discount = 0; voucherCode = ''; voucherApplied = false;
    saveDraftOpen = false;
    activeMobileTab = 'products';
    toast.success('Draft tersimpan', draftName);
  }

  function loadDraft(d: Draft) {
    if ($cart.length > 0 && !confirm('Keranjang saat ini akan diganti. Lanjutkan?')) return;
    cart.clear();
    d.items.forEach(item => cart.addItem(item.product, item.variant, item.qty));
    discount = d.discount;
    discountType = d.discountType;
    voucherCode = d.voucherCode;
    drafts.remove(d.id);
    activePanel = 'none';
    activeMobileTab = 'cart';
    toast.success('Draft dimuat', d.name);
  }

  const statusCfg: Record<string, { label: string; variant: any }> = {
    paid:      { label: 'Lunas',   variant: 'success' },
    pending:   { label: 'Pending', variant: 'warning' },
    refunded:  { label: 'Refund',  variant: 'danger'  },
    cancelled: { label: 'Batal',   variant: 'neutral' },
  };
</script>

<svelte:head><title>Kasir — TiketKu</title></svelte:head>

<div class="pos-shell" 
  class:has-panel={activePanel !== 'none'}
  class:tab-products={activeMobileTab === 'products'}
  class:tab-cart={activeMobileTab === 'cart'}
  class:tab-history={activeMobileTab === 'history'}
  class:tab-drafts={activeMobileTab === 'drafts'}
>
  <aside class="pos-left">
    <div class="pos-search">
      <Search size={15} style="color:var(--text-3);flex-shrink:0;" />
      <input class="pos-search-input" type="search" bind:value={searchQuery} placeholder="Cari produk / SKU..." />
    </div>
    <nav class="pos-cats">
      {#each categories as c}
        <button class="cat-btn" class:active={selectedCategory === c.id} on:click={() => (selectedCategory = c.id)}>
          <span class="cat-label">{c.name}</span>
          <span class="cat-count">{c.id === 'all' ? products.length : products.filter(p => p.category_id === c.id).length}</span>
        </button>
      {/each}
    </nav>
    <div class="pos-actions">
      <button class="action-btn" class:active={activePanel === 'history'} on:click={() => togglePanel('history')}>
        <History size={15} /><span>Riwayat</span>
      </button>
      <button class="action-btn" class:active={activePanel === 'drafts'} on:click={() => togglePanel('drafts')}>
        <Bookmark size={15} /><span>Draft{allDrafts.length > 0 ? ` (${allDrafts.length})` : ''}</span>
      </button>
    </div>

    <!-- Printer Connection Widget -->
    <div class="pos-printer-widget">
      <div class="widget-status">
        {#if $btState === 'connected'}
          <div class="status-indicator success">
            <Bluetooth size={14} />
            <span class="status-text">{$btDevice?.name || 'BLE Printer'}</span>
          </div>
          <button class="widget-disconnect-btn" on:click={() => bluetooth.disconnect()} title="Putuskan BLE">
            <X size={12} />
          </button>
        {:else if $serialState === 'connected'}
          <div class="status-indicator success">
            <Usb size={14} />
            <span class="status-text">{$serialPortName || 'Serial Printer'}</span>
          </div>
          <button class="widget-disconnect-btn" on:click={() => serial.disconnect()} title="Putuskan Serial">
            <X size={12} />
          </button>
        {:else if $bridgeState === 'online'}
          <div class="status-indicator success">
            <Server size={14} />
            <span class="status-text">Bridge: {$bridgePortName || 'Online'}</span>
          </div>
        {:else}
          <div class="status-indicator offline">
            <BluetoothOff size={14} />
            <span class="status-text">Printer Terputus</span>
          </div>
        {/if}
      </div>

      {#if $btState !== 'connected' && $serialState !== 'connected'}
        <div class="widget-actions">
          <button class="widget-btn ble" on:click={() => bluetooth.connect()} disabled={$btState === 'connecting'}>
            {#if $btState === 'connecting'}
              <Spinner size={10} color="var(--brand-600)" /> <span>BLE...</span>
            {:else}
              <Bluetooth size={12} /> <span>BLE</span>
            {/if}
          </button>
          <button class="widget-btn serial" on:click={() => serial.connect()} disabled={$serialState === 'connecting'}>
            {#if $serialState === 'connecting'}
              <Spinner size={10} color="var(--brand-600)" /> <span>Serial...</span>
            {:else}
              <Usb size={12} /> <span>Serial</span>
            {/if}
          </button>
        </div>
      {/if}
    </div>

    <button class="reload-btn" on:click={loadAll} disabled={loadingProducts} aria-label="Refresh">
      <RefreshCw size={13} />
    </button>
  </aside>

  <main class="pos-main">
    <!-- Mobile-only search bar -->
    <div class="mobile-search-bar">
      <Search size={15} style="color:var(--text-3);flex-shrink:0;" />
      <input class="mobile-search-input" type="search" bind:value={searchQuery} placeholder="Cari produk / SKU..." />
      <button class="reload-btn" style="margin:0;padding:4px;" on:click={loadAll} disabled={loadingProducts} aria-label="Refresh">
        <RefreshCw size={13} />
      </button>
    </div>

    <!-- Mobile-only Printer Connection Widget -->
    <div class="mobile-printer-bar">
      <div class="mp-status">
        {#if $btState === 'connected'}
          <span class="status-indicator success" style="display:flex;align-items:center;gap:4px;font-size:0.75rem;font-weight:500;color:var(--emerald);">
            <Bluetooth size={13} /> {$btDevice?.name || 'BLE Printer'}
          </span>
          <button class="widget-disconnect-btn" on:click={() => bluetooth.disconnect()}><X size={12} /></button>
        {:else if $serialState === 'connected'}
          <span class="status-indicator success" style="display:flex;align-items:center;gap:4px;font-size:0.75rem;font-weight:500;color:var(--emerald);">
            <Usb size={13} /> {$serialPortName || 'Serial Printer'}
          </span>
          <button class="widget-disconnect-btn" on:click={() => serial.disconnect()}><X size={12} /></button>
        {:else if $bridgeState === 'online'}
          <span class="status-indicator success" style="display:flex;align-items:center;gap:4px;font-size:0.75rem;font-weight:500;color:var(--emerald);">
            <Server size={13} /> Bridge: {$bridgePortName || 'Online'}
          </span>
        {:else}
          <span class="status-indicator offline" style="display:flex;align-items:center;gap:4px;font-size:0.75rem;font-weight:500;color:var(--text-3);">
            <BluetoothOff size={13} /> Printer Terputus
          </span>
        {/if}
      </div>

      {#if $btState !== 'connected' && $serialState !== 'connected'}
        <div class="mp-actions">
          <button class="mp-btn ble" on:click={() => bluetooth.connect()} disabled={$btState === 'connecting'}>
            {#if $btState === 'connecting'}
              <Spinner size={10} color="var(--brand-600)" /> <span style="font-size:0.6875rem;">BLE...</span>
            {:else}
              <Bluetooth size={12} /> <span style="font-size:0.6875rem;font-weight:600;">BLE</span>
            {/if}
          </button>
          <button class="mp-btn serial" on:click={() => serial.connect()} disabled={$serialState === 'connecting'}>
            {#if $serialState === 'connecting'}
              <Spinner size={10} color="var(--brand-600)" /> <span style="font-size:0.6875rem;">Serial...</span>
            {:else}
              <Usb size={12} /> <span style="font-size:0.6875rem;font-weight:600;">Serial</span>
            {/if}
          </button>
        </div>
      {/if}
    </div>

    <!-- Mobile-only category slider -->
    <nav class="mobile-category-bar">
      {#each categories as c}
        <button class="mobile-cat-btn" class:active={selectedCategory === c.id} on:click={() => (selectedCategory = c.id)}>
          {c.name} <span style="opacity:0.7; font-size:0.6875rem;">({c.id === 'all' ? products.length : products.filter(p => p.category_id === c.id).length})</span>
        </button>
      {/each}
    </nav>

    {#if loadingProducts}
      <div class="loading-grid">
        {#each Array(8) as _}<div class="skel" style="height:140px;border-radius:var(--r-lg);"></div>{/each}
      </div>
    {:else}
      <ProductGrid products={filteredByCategory} {searchQuery} selectedCategory="all" on:add={handleAdd} />
    {/if}
  </main>

  {#if activePanel !== 'none'}
    <div class="side-panel">
      <div class="panel-head">
        <div class="panel-title">
          {#if activePanel === 'history'}
            <History size={15} style="color:var(--brand-500);" /><span>Riwayat Transaksi</span>
          {:else}
            <Bookmark size={15} style="color:var(--brand-500);" /><span>Draft Tersimpan</span>
          {/if}
        </div>
        <div style="display:flex;gap:4px;">
          {#if activePanel === 'history'}
            <button class="btn btn-ghost btn-icon btn-sm" on:click={loadHistory} disabled={loadingHistory}><RefreshCw size={13} /></button>
          {:else}
            <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={openSaveDraft}><BookmarkPlus size={13} /> Simpan</button>
          {/if}
          <button class="btn btn-ghost btn-icon btn-sm" on:click={() => (activePanel = 'none')} aria-label="Tutup"><X size={14} /></button>
        </div>
      </div>
      <div class="panel-body">
        {#if activePanel === 'history'}
          {#if loadingHistory}
            <div style="padding:12px;display:flex;flex-direction:column;gap:8px;">
              {#each Array(5) as _}<div class="skel" style="height:52px;border-radius:var(--r-md);"></div>{/each}
            </div>
          {:else if txHistory.length === 0}
            <div class="panel-empty"><Clock size={28} /><p>Belum ada transaksi</p></div>
          {:else}
            {#each txHistory as tx (tx.id)}
              <button class="history-item" on:click={() => openHistoryDetail(tx)}>
                <div class="hi-top">
                  <span class="mono" style="font-size:0.8125rem;font-weight:600;">{tx.invoice}</span>
                  <Badge variant={statusCfg[tx.status]?.variant ?? 'neutral'} size="sm">{statusCfg[tx.status]?.label ?? tx.status}</Badge>
                </div>
                <div class="hi-bot">
                  <span style="font-size:0.8125rem;color:var(--text-2);">{relativeTime(tx.created_at)}</span>
                  <span style="font-weight:600;color:var(--brand-600);">{currency(tx.total)}</span>
                </div>
              </button>
            {/each}
          {/if}
        {:else}
          {#if allDrafts.length === 0}
            <div class="panel-empty"><Bookmark size={28} /><p>Belum ada draft</p><span>Klik &ldquo;Simpan&rdquo; untuk menyimpan keranjang</span></div>
          {:else}
            {#each allDrafts as d (d.id)}
              <div class="draft-item">
                <div class="di-info">
                  <span class="di-name">{d.name}</span>
                  <span class="di-meta">{d.items.length} item &bull; {relativeTime(d.savedAt)}</span>
                </div>
                <div style="display:flex;gap:4px;">
                  <button class="btn btn-primary btn-sm" style="gap:4px;" on:click={() => loadDraft(d)}><RotateCcw size={12} /> Muat</button>
                  <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => { drafts.remove(d.id); toast.info('Draft dihapus', d.name); }} aria-label="Hapus"><Trash2 size={13} /></button>
                </div>
              </div>
            {/each}
          {/if}
        {/if}
      </div>
    </div>
  {/if}

  <CartPanel bind:discount bind:discountType bind:voucherCode bind:voucherApplied
    on:checkout={() => { if ($cartSummary.qty === 0) { toast.warning('Keranjang kosong', ''); return; } checkoutOpen = true; }}
    on:voucherApply={(e) => { voucherApplied = true; toast.success('Voucher aktif', e.detail); }}
  />

  <!-- Mobile Bottom Navigation Bar -->
  <div class="pos-mobile-nav">
    <button class="mobile-nav-btn" class:active={activeMobileTab === 'products'} on:click={() => setMobileTab('products')}>
      <Search size={18} />
      <span>Produk</span>
    </button>
    <button class="mobile-nav-btn" class:active={activeMobileTab === 'cart'} on:click={() => setMobileTab('cart')}>
      <div style="position:relative; display:flex; align-items:center; justify-content:center;">
        <ShoppingCart size={18} />
        {#if $cartSummary.qty > 0}
          <span class="mobile-badge">{$cartSummary.qty}</span>
        {/if}
      </div>
      <span>Keranjang</span>
    </button>
    <button class="mobile-nav-btn" class:active={activeMobileTab === 'history'} on:click={() => setMobileTab('history')}>
      <History size={18} />
      <span>Riwayat</span>
    </button>
    <button class="mobile-nav-btn" class:active={activeMobileTab === 'drafts'} on:click={() => setMobileTab('drafts')}>
      <div style="position:relative; display:flex; align-items:center; justify-content:center;">
        <Bookmark size={18} />
        {#if allDrafts.length > 0}
          <span class="mobile-badge" style="background:var(--brand-500);">{allDrafts.length}</span>
        {/if}
      </div>
      <span>Draft</span>
    </button>
  </div>
</div>

<Modal bind:open={variantModalOpen} title="Pilih Varian" size="sm" on:close={closeVariantModal}>
  {#if selectedProduct}
    <div class="modal-body">
      <p style="color:var(--text-2);font-size:0.875rem;margin-bottom:14px;">{selectedProduct.name}</p>
      {#if loadingVariants}
        <div style="display:flex;flex-direction:column;gap:8px;">{#each Array(3) as _}<div class="skel" style="height:48px;border-radius:var(--r-md);"></div>{/each}</div>
      {:else if selectedVariants.length === 0}
        <div style="text-align:center;padding:24px;color:var(--text-3);font-size:0.875rem;">
          Tidak ada varian aktif.
          <button class="btn btn-secondary btn-sm" style="margin-top:12px;display:block;"
            on:click={() => { if (selectedProduct) { cart.addItem(selectedProduct); variantModalOpen = false; toast.success('Ditambahkan', selectedProduct.name); } }}>
            Tambah tanpa varian
          </button>
        </div>
      {:else}
        <div style="display:flex;flex-direction:column;gap:8px;">
          {#each selectedVariants as v}
            <button class="var-opt" on:click={() => addVariant(v)} disabled={v.status === 'inactive'}>
              <div style="flex:1;text-align:left;"><span style="font-weight:600;">{v.name}</span>
                {#if v.quota !== null && v.quota !== undefined}<span style="font-size:0.75rem;color:var(--text-3);display:block;">Sisa: {(v.quota ?? 0) - (v.quota_used ?? 0)}</span>{/if}
              </div>
              <span style="font-weight:700;color:var(--brand-600);">{currency(v.price)}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</Modal>

<CheckoutModal bind:open={checkoutOpen} {discountAmt} {total} {loading}
  on:confirm={handleConfirm} on:close={closeCheckout} />

<Modal bind:open={successOpen} title="" size="sm">
  <div class="modal-body" style="text-align:center;padding:32px 24px;">
    <svg width="56" height="56" viewBox="0 0 56 56" fill="none" aria-hidden="true" style="margin:0 auto;">
      <circle cx="28" cy="28" r="28" fill="rgba(16,185,129,0.12)"/>
      <path d="M18 28l8 8 12-14" stroke="var(--emerald)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    <h2 style="font-size:1.25rem;margin:16px 0 4px;">Pembayaran Berhasil!</h2>
    <p class="mono" style="color:var(--text-2);">{lastInvoice}</p>
    <div style="display:flex;gap:10px;justify-content:center;margin-top:24px;flex-wrap:wrap;">
      <button class="btn btn-secondary" style="gap:6px;" on:click={() => openPrintModal(lastTxId)}><Printer size={14} /> Cetak</button>
      <button class="btn btn-primary" on:click={() => (successOpen = false)}>Transaksi Baru</button>
    </div>
  </div>
</Modal>

<Modal bind:open={historyDetailOpen} title="Detail Transaksi" size="md">
  {#if historyDetail}
    <div class="modal-body">
      <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:16px;flex-wrap:wrap;gap:8px;">
        <div>
          <span class="mono" style="font-size:1rem;font-weight:700;">{historyDetail.transaction?.invoice}</span>
          <span style="display:block;font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{historyDetail.transaction?.created_at ? datetime(historyDetail.transaction.created_at) : ''}</span>
        </div>
        <Badge variant={statusCfg[historyDetail.transaction?.status]?.variant ?? 'neutral'}>{statusCfg[historyDetail.transaction?.status]?.label ?? historyDetail.transaction?.status}</Badge>
      </div>
      <div class="tx-items">
        {#each historyDetail.items ?? [] as item}
          <div class="tx-item-row"><span style="flex:1;">{item.product_name}</span><span style="color:var(--text-2);">x{item.qty}</span><span style="font-weight:500;min-width:80px;text-align:right;">{currency(item.subtotal)}</span></div>
        {/each}
      </div>
      {#if historyTickets.length > 0}
        <div style="margin-top: 14px; border-top: 1px solid var(--border); padding-top: 10px; margin-bottom: 12px;">
          <span style="font-size: 0.8125rem; font-weight: 600; color: var(--text-2); display: block; margin-bottom: 6px;">Daftar Tiket:</span>
          <div style="display: flex; flex-direction: column; gap: 4px;">
            {#each historyTickets as tk}
              <div style="display: flex; justify-content: space-between; align-items: center; font-size: 0.8125rem; padding: 4px 8px; background: var(--bg-subtle); border-radius: var(--r-sm);">
                <div>
                  <span class="mono" style="font-weight: 600;">{tk.code}</span>
                  <span style="color: var(--text-3); margin-left: 6px;">({tk.product_name})</span>
                </div>
                <span style="font-weight: bold; background: var(--brand-50); color: var(--brand-600); padding: 1px 6px; border-radius: var(--r-full); font-size: 0.75rem;">Urut: #{tk.queue_number ?? 1}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
      <div class="tx-summary">
        <div class="tx-sum-row"><span>Subtotal</span><span>{currency(historyDetail.transaction?.subtotal ?? 0)}</span></div>
        {#if (historyDetail.transaction?.discount ?? 0) > 0}
          <div class="tx-sum-row" style="color:var(--rose);"><span>Diskon</span><span>- {currency(historyDetail.transaction.discount)}</span></div>
        {/if}
        <div class="tx-sum-row total"><span>Total</span><span>{currency(historyDetail.transaction?.total ?? 0)}</span></div>
        {#each historyDetail.payments ?? [] as p}
          <div class="tx-sum-row" style="color:var(--text-2);font-size:0.8125rem;"><span style="text-transform:capitalize;">{p.method}</span><span>{currency(p.amount)}</span></div>
        {/each}
        {#if (historyDetail.transaction?.change ?? 0) > 0}
          <div class="tx-sum-row" style="color:var(--emerald);"><span>Kembalian</span><span>{currency(historyDetail.transaction.change)}</span></div>
        {/if}
      </div>
    </div>
    <div class="modal-footer">
      {#if historyDetail.transaction?.status === 'paid'}
        <button class="btn btn-secondary" style="gap:6px;" on:click={() => confirmRefund(historyDetail.transaction.id, historyDetail.transaction.invoice)}><RotateCcw size={14} /> Refund</button>
        <button class="btn btn-primary" style="gap:6px;" on:click={() => { openPrintModal(historyDetail.transaction.id); historyDetailOpen = false; }}><Printer size={14} /> Cetak Ulang</button>
      {:else}
        <button class="btn btn-secondary" on:click={() => (historyDetailOpen = false)}>Tutup</button>
      {/if}
    </div>
  {/if}
</Modal>

<!-- Refund Transaction Confirmation Modal -->
<Modal bind:open={refundOpen} title="Refund Transaksi" size="sm">
  <div class="modal-body">
    <p>Yakin ingin merefund transaksi <strong>{refundTarget?.invoice}</strong>? Tindakan ini tidak dapat dibatalkan.</p>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (refundOpen = false)}>Batal</button>
    <button class="btn btn-danger" on:click={executeRefund} disabled={refunding}>
      {#if refunding}<Spinner size={14} color="#fff" />{/if} Refund
    </button>
  </div>
</Modal>

<Modal bind:open={saveDraftOpen} title="Simpan sebagai Draft" size="sm">
  <div class="modal-body">
    <div class="field">
      <label class="label" for="draft-name">Nama Draft</label>
      <input id="draft-name" class="input" bind:value={draftName} placeholder="Nama pelanggan / keterangan" />
      <span class="field-hint">{$cart.length} item &bull; Total {currency(total)}</span>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (saveDraftOpen = false)}>Batal</button>
    <button class="btn btn-primary" style="gap:6px;" on:click={saveDraftConfirm} disabled={!draftName.trim()}>
      <BookmarkPlus size={14} /> Simpan Draft
    </button>
  </div>
</Modal>

<style>
  .pos-shell { display:grid; grid-template-columns:185px 1fr 300px; height:calc(100dvh - var(--topbar-h)); margin:-24px; overflow:hidden; }
  .pos-shell.has-panel { grid-template-columns:185px 1fr 260px 300px; }
  .pos-left { background:var(--bg-surface); border-right:1px solid var(--border); display:flex; flex-direction:column; overflow:hidden; }
  .pos-search { display:flex; align-items:center; gap:8px; padding:10px 12px; border-bottom:1px solid var(--border); }
  .pos-search-input { flex:1; border:none; outline:none; background:transparent; font-family:var(--font-sans); font-size:0.875rem; color:var(--text-1); }
  .pos-search-input::placeholder { color:var(--text-3); }
  .pos-cats { display:flex; flex-direction:column; padding:8px; gap:2px; overflow-y:auto; flex:1; }
  .cat-btn { display:flex; align-items:center; justify-content:space-between; padding:8px 10px; border-radius:var(--r-md); font-size:0.875rem; font-weight:500; color:var(--text-2); cursor:pointer; transition:background var(--ease-fast),color var(--ease-fast); border:none; background:none; font-family:var(--font-sans); width:100%; }
  .cat-btn:hover { background:var(--bg-muted); color:var(--text-1); }
  .cat-btn.active { background:var(--brand-50); color:var(--brand-600); font-weight:600; }
  [data-theme='dark'] .cat-btn.active { background:rgba(99,102,241,0.14); color:var(--brand-300); }
  .cat-label { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .cat-count { font-size:0.6875rem; font-weight:600; background:var(--bg-muted); color:var(--text-3); padding:1px 6px; border-radius:var(--r-full); flex-shrink:0; }
  .pos-actions { padding:6px 8px; border-top:1px solid var(--border); display:flex; flex-direction:column; gap:2px; }
  .action-btn { display:flex; align-items:center; gap:8px; padding:7px 10px; border-radius:var(--r-md); font-size:0.8125rem; font-weight:500; color:var(--text-2); cursor:pointer; border:none; background:none; font-family:var(--font-sans); width:100%; transition:all var(--ease-fast); }
  .action-btn:hover { background:var(--bg-muted); color:var(--text-1); }
  .action-btn.active { background:var(--brand-50); color:var(--brand-600); font-weight:600; }
  [data-theme='dark'] .action-btn.active { background:rgba(99,102,241,0.14); color:var(--brand-300); }
  .reload-btn { margin:8px; padding:6px; display:flex; align-items:center; justify-content:center; border:none; background:none; cursor:pointer; color:var(--text-3); border-radius:var(--r-md); transition:all var(--ease-fast); font-family:var(--font-sans); }
  .reload-btn:hover { background:var(--bg-muted); color:var(--text-1); }
  .pos-main { overflow-y:auto; }
  .loading-grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(155px,1fr)); gap:12px; padding:16px; }
  /* Side Panel */
  .side-panel { background:var(--bg-surface); border-left:1px solid var(--border); display:flex; flex-direction:column; overflow:hidden; }
  .panel-head { display:flex; align-items:center; justify-content:space-between; padding:12px 14px; border-bottom:1px solid var(--border); flex-shrink:0; }
  .panel-title { display:flex; align-items:center; gap:6px; font-size:0.875rem; font-weight:600; color:var(--text-1); flex:1; }
  .panel-body { flex:1; overflow-y:auto; }
  .panel-empty { display:flex; flex-direction:column; align-items:center; justify-content:center; gap:8px; padding:32px 16px; color:var(--text-3); text-align:center; height:200px; }
  .panel-empty p { font-size:0.875rem; font-weight:500; color:var(--text-2); }
  .panel-empty span { font-size:0.8125rem; }
  /* History */
  .history-item { display:flex; flex-direction:column; gap:5px; width:100%; padding:10px 14px; border-bottom:1px solid var(--border-subtle); cursor:pointer; background:none; border-left:none; border-right:none; border-top:none; font-family:var(--font-sans); text-align:left; transition:background var(--ease-fast); }
  .history-item:hover { background:var(--bg-subtle); }
  .hi-top { display:flex; align-items:center; justify-content:space-between; gap:8px; }
  .hi-bot { display:flex; align-items:center; justify-content:space-between; }
  /* Draft */
  .draft-item { display:flex; align-items:center; gap:10px; padding:10px 14px; border-bottom:1px solid var(--border-subtle); }
  .di-info { flex:1; min-width:0; }
  .di-name { font-size:0.875rem; font-weight:600; color:var(--text-1); display:block; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
  .di-meta { font-size:0.75rem; color:var(--text-3); display:block; margin-top:2px; }
  /* Tx detail */
  .tx-items { display:flex; flex-direction:column; gap:6px; margin-bottom:12px; }
  .tx-item-row { display:flex; align-items:center; gap:10px; font-size:0.875rem; padding:4px 0; border-bottom:1px solid var(--border-subtle); }
  .tx-summary { background:var(--bg-subtle); border-radius:var(--r-md); padding:10px 14px; display:flex; flex-direction:column; gap:5px; }
  .tx-sum-row { display:flex; justify-content:space-between; font-size:0.875rem; color:var(--text-2); }
  .tx-sum-row.total { font-size:1rem; font-weight:700; color:var(--text-1); padding-top:6px; border-top:1px solid var(--border); margin-top:2px; }
  /* Variant */
  .var-opt { display:flex; align-items:center; justify-content:space-between; padding:12px 14px; border-radius:var(--r-md); border:1px solid var(--border); background:var(--bg-surface); cursor:pointer; font-family:var(--font-sans); transition:border-color var(--ease-fast),background var(--ease-fast),transform var(--ease-fast); gap:12px; width:100%; }
  .var-opt:hover { border-color:var(--brand-400); background:var(--brand-50); transform:translateY(-1px); }
  [data-theme='dark'] .var-opt:hover { background:rgba(99,102,241,0.1); }
  .var-opt:disabled { opacity:0.4; cursor:not-allowed; transform:none; }
  .pos-mobile-nav { display: none; }
  .mobile-search-bar { display: none; }
  .mobile-category-bar { display: none; }

  @media (max-width:1100px) { .pos-shell.has-panel { grid-template-columns:185px 1fr 220px 280px; } }
  
  @media (max-width: 900px) {
    .pos-shell, .pos-shell.has-panel {
      grid-template-columns: 1fr !important;
      height: calc(100dvh - var(--topbar-h) - 60px - env(safe-area-inset-bottom, 0px)) !important;
      margin: -16px !important;
      position: relative;
    }
    
    .pos-left {
      display: none !important;
    }
    
    /* Toggle components visibility on mobile */
    .pos-shell:not(.tab-products) .pos-main {
      display: none !important;
    }
    
    .pos-shell:not(.tab-cart) :global(.cart-panel) {
      display: none !important;
    }
    
    .pos-shell:not(.tab-history):not(.tab-drafts) .side-panel {
      display: none !important;
    }

    .pos-main, :global(.cart-panel), .side-panel {
      width: 100% !important;
      height: 100% !important;
      border: none !important;
    }

    /* Mobile category slider */
    .mobile-category-bar {
      display: flex;
      gap: 6px;
      overflow-x: auto;
      padding: 10px 16px;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border);
      scrollbar-width: none;
      -ms-overflow-style: none;
    }
    .mobile-category-bar::-webkit-scrollbar {
      display: none;
    }
    .mobile-cat-btn {
      padding: 6px 12px;
      border-radius: var(--r-full);
      background: var(--bg-subtle);
      border: 1px solid var(--border-subtle);
      font-size: 0.8125rem;
      font-weight: 500;
      color: var(--text-2);
      white-space: nowrap;
      cursor: pointer;
      font-family: var(--font-sans);
      transition: all var(--ease-fast);
    }
    .mobile-cat-btn.active {
      background: var(--brand-500);
      color: white;
      border-color: var(--brand-500);
    }

    /* Mobile search bar */
    .mobile-search-bar {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 10px 16px;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border);
    }
    .mobile-search-input {
      flex: 1;
      border: none;
      outline: none;
      background: transparent;
      font-family: var(--font-sans);
      font-size: 0.875rem;
      color: var(--text-1);
    }

    /* Mobile Bottom Navigation */
    .pos-mobile-nav {
      display: flex;
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      height: calc(60px + env(safe-area-inset-bottom, 0px));
      padding-bottom: env(safe-area-inset-bottom, 0px);
      background: var(--bg-surface);
      border-top: 1px solid var(--border);
      z-index: 40;
      justify-content: space-around;
      align-items: center;
    }
    .mobile-nav-btn {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 4px;
      color: var(--text-3);
      background: none;
      border: none;
      font-size: 0.75rem;
      font-weight: 500;
      font-family: var(--font-sans);
      flex: 1;
      height: 100%;
      cursor: pointer;
      position: relative;
      transition: color var(--ease-fast);
    }
    .mobile-nav-btn.active {
      color: var(--brand-600);
    }
    [data-theme='dark'] .mobile-nav-btn.active {
      color: var(--brand-400);
    }
    .mobile-badge {
      position: absolute;
      top: -6px;
      right: -10px;
      background: var(--rose);
      color: white;
      font-size: 0.625rem;
      font-weight: 700;
      min-width: 15px;
      height: 15px;
      padding: 0 4px;
      border-radius: var(--r-full);
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }

  /* Sidebar Printer Connection Widget */
  .pos-printer-widget {
    margin: 8px;
    padding: 10px;
    background: var(--bg-subtle);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .widget-status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    font-weight: 500;
  }
  .status-indicator.success {
    color: var(--emerald);
  }
  .status-indicator.offline {
    color: var(--text-3);
  }
  .status-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
  }
  .widget-disconnect-btn {
    border: none;
    background: none;
    cursor: pointer;
    color: var(--text-3);
    padding: 2px;
    border-radius: var(--r-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background var(--ease-fast), color var(--ease-fast);
  }
  .widget-disconnect-btn:hover {
    background: var(--bg-muted);
    color: var(--rose);
  }
  .widget-actions {
    display: flex;
    gap: 6px;
  }
  .widget-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 5px 6px;
    border-radius: var(--r-sm);
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--bg-surface);
    color: var(--text-2);
    transition: all var(--ease-fast);
    font-family: var(--font-sans);
  }
  .widget-btn:hover {
    background: var(--bg-muted);
    color: var(--text-1);
    border-color: var(--text-3);
  }
  .widget-btn.ble:hover {
    background: rgba(99,102,241,0.06);
    color: var(--brand-600);
    border-color: var(--brand-300);
  }
  .widget-btn.serial:hover {
    background: rgba(16,185,129,0.06);
    color: var(--emerald);
    border-color: var(--emerald-300);
  }

  /* Mobile Printer connection bar */
  .mobile-printer-bar {
    display: none;
  }
  
  @media (max-width: 900px) {
    .mobile-printer-bar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 6px 16px;
      background: var(--bg-subtle);
      border-bottom: 1px solid var(--border);
      gap: 12px;
      flex-shrink: 0;
    }
    .mp-status {
      display: flex;
      align-items: center;
      justify-content: space-between;
      flex: 1;
      min-width: 0;
    }
    .mp-actions {
      display: flex;
      gap: 6px;
      flex-shrink: 0;
    }
    .mp-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      padding: 4px 8px;
      border-radius: var(--r-sm);
      cursor: pointer;
      border: 1px solid var(--border);
      background: var(--bg-surface);
      color: var(--text-2);
      font-family: var(--font-sans);
      transition: all var(--ease-fast);
    }
    .mp-btn:hover {
      background: var(--bg-muted);
      color: var(--text-1);
    }
    .mp-btn.ble:hover {
      background: rgba(99,102,241,0.06);
      color: var(--brand-600);
      border-color: var(--brand-300);
    }
    .mp-btn.serial:hover {
      background: rgba(16,185,129,0.06);
      color: var(--emerald);
      border-color: var(--emerald-300);
    }
  }
</style>

<!-- Print Modal -->
<PrintModal
  bind:open={printModalOpen}
  transaction={printTxData}
  items={printItems}
  payments={printPayments}
  tickets={printTickets}
  venueName={venueSettings.name}
  venueAddress={venueSettings.address}
  venuePhone={venueSettings.phone}
  venueEmail={venueSettings.email}
  venueFooter1={venueFooter1}
  venueFooter2={venueFooter2}
  paperWidth={paperWidth}
  qrEnabled={qrEnabled}
  on:close={() => (printModalOpen = false)}
/>
