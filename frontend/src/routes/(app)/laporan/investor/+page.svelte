<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  import { onMount, afterUpdate } from 'svelte';
  import { currency, number } from '$lib/utils/format';
  import { api } from '$lib/api/client';
  import { toast } from '$lib/stores/toast';
  import { userRole } from '$lib/stores/auth';
  import { TrendingUp, Download, Calendar, Filter, Users, Package } from 'lucide-svelte';

  let loading = true;
  let dateFrom = new Date(Date.now() - 30 * 86400000).toISOString().slice(0,10);
  let dateTo   = new Date().toISOString().slice(0,10);
  let investors: any[] = [];
  let products: any[] = [];
  let reportData: any[] = [];
  let selectedInvestorId = '';
  let selectedProductId = '';
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let chartInitialized = false;

  $: isAdmin = $userRole === 'admin';
  $: totalRevenue = reportData.reduce((s, d) => s + (d.revenue ?? 0), 0);
  $: investorRevenue = reportData.reduce((s, d) => s + (d.investor_share ?? 0), 0);

  async function loadInvestors() {
    if (!isAdmin) return;
    try {
      const res = await api.get<any[]>('/investors');
      investors = (res as any)?.data ?? [];
    } catch { investors = []; }
  }

  async function loadProducts() {
    try {
      const res = await api.get<any[]>('/products?per_page=200');
      products = (res as any)?.data ?? [];
    } catch { products = []; }
  }

  async function loadData() {
    loading = true;
    chartInitialized = false;
    try {
      const params: Record<string, string> = { from: dateFrom, to: dateTo };
      if (isAdmin && selectedInvestorId) params.investor_id = selectedInvestorId;
      if (selectedProductId) params.product_id = selectedProductId;
      const res = await api.get<any[]>(`/reports/investors?${new URLSearchParams(params)}`);
      const d = (res as any)?.data;
      reportData = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat laporan investor', e.message);
      reportData = [];
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await Promise.all([loadInvestors(), loadProducts()]);
    await loadData();
  });

  afterUpdate(() => {
    if (!loading && chartCanvas && reportData.length > 0 && !chartInitialized) {
      chartInitialized = true;
      initChart();
    }
  });

  function initChart() {
    if (!chartCanvas || reportData.length === 0) return;
    import('chart.js/auto').then(({ default: Chart }) => {
      if (chart) chart.destroy();
      const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
      const textColor = isDark ? '#9898a6' : '#6e6e80';
      const gridColor = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      chart = new Chart(chartCanvas, {
        type: 'bar',
        data: {
          labels: reportData.map(d => d.product_name ?? '-'),
          datasets: [
            { label: 'Total Revenue', data: reportData.map(d => d.revenue ?? 0), backgroundColor: 'rgba(99,102,241,0.7)', borderRadius: 4 },
            { label: 'Investor Share', data: reportData.map(d => d.investor_share ?? 0), backgroundColor: 'rgba(16,185,129,0.7)', borderRadius: 4 },
          ]
        },
        options: {
          responsive: true, maintainAspectRatio: false,
          plugins: {
            legend: { position: 'top', labels: { color: textColor, font: { size: 12 } } },
            tooltip: { callbacks: { label: (ctx: any) => `${ctx.dataset.label}: ${currency(ctx.parsed.y ?? 0)}` } }
          },
          scales: {
            x: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 11 } } },
            y: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 11 }, callback: (v: any) => 'Rp' + Number(v) / 1000000 + 'jt' } }
          }
        }
      });
    });
  }
</script>

<svelte:head><title>Laporan Investor — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Laporan Investor</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Pendapatan per investor dan produk</p>
    </div>
    <button class="btn btn-secondary" style="gap:6px;"><Download size={14} /> Export</button>
  </div>

  <!-- Filter -->
  <div class="card" style="padding:14px 18px;">
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
      <Calendar size={16} style="color:var(--text-3);" />
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="if">Dari</label>
        <input id="if" class="input input-sm" type="date" bind:value={dateFrom} style="width:150px;" />
      </div>
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="it">Sampai</label>
        <input id="it" class="input input-sm" type="date" bind:value={dateTo} style="width:150px;" />
      </div>
      {#if isAdmin && investors.length > 0}
        <select class="input input-sm" style="width:180px;" bind:value={selectedInvestorId} on:change={() => selectedProductId = ''}>
          <option value="">Semua Investor</option>
          {#each investors as inv}<option value={inv.id}>{inv.name}</option>{/each}
        </select>
      {/if}
      {#if products.length > 0}
        <select class="input input-sm" style="width:180px;" bind:value={selectedProductId}>
          <option value="">Semua Produk</option>
          {#each products.filter(p => !isAdmin || !selectedInvestorId || p.investor_id === selectedInvestorId) as prod}
            <option value={prod.id}>{prod.name}</option>
          {/each}
        </select>
      {/if}
      <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={loadData} disabled={loading}>
        <Filter size={13} /> {loading ? 'Memuat...' : 'Terapkan'}
      </button>
    </div>
  </div>

  {#if !isAdmin && products.length === 0}
    <div class="empty-state" style="margin-top:20px;">
      <Package size={48} style="margin:0 auto 16px;opacity:0.3;color:var(--brand-500);" />
      <p style="font-size:1.1rem;font-weight:600;color:var(--text-1);margin-bottom:4px;">Belum Ada Produk Terkait</p>
      <span style="font-size:0.875rem;color:var(--text-3);max-width:400px;line-height:1.5;">Akun Anda belum dikaitkan dengan produk investasi apa pun. Silakan hubungi Administrator untuk menghubungkan produk Anda.</span>
    </div>
  {:else}
    <div class="stats-row">
      <div class="stat-mini card"><span class="stat-lbl">Total Revenue</span><span class="stat-val" style="color:var(--brand-500)">{loading ? '...' : currency(totalRevenue)}</span></div>
      <div class="stat-mini card"><span class="stat-lbl">Share Investor</span><span class="stat-val" style="color:var(--emerald)">{loading ? '...' : currency(investorRevenue)}</span></div>
      <div class="stat-mini card"><span class="stat-lbl">Produk</span><span class="stat-val" style="color:var(--amber)">{reportData.length}</span></div>
    </div>

    {#if loading}
      <div style="display:flex;flex-direction:column;gap:10px;margin-top:20px;">{#each Array(4) as _}<div class="skel" style="height:48px;"></div>{/each}</div>
    {:else if reportData.length === 0}
      <div class="empty-state" style="margin-top:20px;">
        <Users size={32} style="margin:0 auto 12px;opacity:0.3;" />
        {#if !isAdmin}
          <p>Tidak ada data penjualan untuk produk Anda dalam periode ini</p>
          <span style="font-size:0.875rem;color:var(--text-3);">Cobalah ubah filter rentang tanggal pencarian Anda</span>
        {:else}
          <p>Tidak ada data investor untuk periode ini</p>
          <span style="font-size:0.875rem;color:var(--text-3);">Pastikan produk sudah dikaitkan dengan investor di Master Data</span>
        {/if}
      </div>
    {:else}
      <div class="card" style="padding:20px;margin-top:20px;">
        <h3 style="margin-bottom:16px;">Revenue vs Investor Share per Produk</h3>
        <div style="height:300px;"><canvas bind:this={chartCanvas}></canvas></div>
      </div>

      <div class="table-wrap" style="margin-top:20px;">
        <table class="table">
          <thead>
            <tr>
              <th>Produk</th>
              <th>Investor</th>
              <th>Qty Terjual</th>
              <th>Total Revenue</th>
              <th>Fee %</th>
              <th>Share Investor</th>
            </tr>
          </thead>
          <tbody>
            {#each reportData as row}
              <tr>
                <td style="font-weight:600;">{row.product_name ?? '-'}</td>
                <td style="color:var(--text-2);">{row.investor_name ?? '-'}</td>
                <td>{number(row.qty_sold ?? 0)}</td>
                <td style="font-weight:500;">{currency(row.revenue ?? 0)}</td>
                <td style="color:var(--text-2);">{row.revenue_pct ?? 0}%</td>
                <td style="font-weight:700;color:var(--emerald);">{currency(row.investor_share ?? 0)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1100px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .stats-row { display:grid;grid-template-columns:repeat(3,1fr);gap:14px; }
  .stat-mini { padding:18px 20px; }
  .stat-lbl { font-size:0.75rem;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:var(--text-2);display:block;margin-bottom:8px; }
  .stat-val { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em;display:block; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg);display:flex;flex-direction:column;align-items:center;gap:8px; }
  .empty-state p { font-size:0.9375rem;font-weight:500;color:var(--text-2); }
  @media (max-width:640px) { .stats-row { grid-template-columns:1fr; } }
</style>
