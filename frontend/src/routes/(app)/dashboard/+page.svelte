<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { currency, number, relativeTime } from '$lib/utils/format';
  import { btState } from '$lib/stores/bluetooth';
  import { dashboardApi, transactionsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import StatCard from '$lib/components/ui/StatCard.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import {
    ShoppingCart, Scan, RefreshCw, Plus, Printer,
    Ticket, TrendingUp, ArrowUpRight, Activity
  } from 'lucide-svelte';
  import type { DashboardStats, Transaction } from '$lib/types';

  let stats: DashboardStats | null = null;
  let recentTx: Transaction[] = [];
  let loading = true;
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let ChartClass: any = null;
  let debugError: string | null = null;

  async function loadData() {
    loading = true;
    try {
      const [statsRes, txRes] = await Promise.all([
        dashboardApi.stats(),
        transactionsApi.list({ per_page: '5' }),
      ]);
      const statsData = (statsRes as any)?.data ?? {};
      const txData    = (txRes as any)?.data;
      recentTx = Array.isArray(txData) ? txData : [];

      // Demo chart data — shown when API has no real data yet
      const demoSalesChart = [
        {hour:'08',revenue:450000},{hour:'09',revenue:980000},{hour:'10',revenue:1200000},
        {hour:'11',revenue:1800000},{hour:'12',revenue:2100000},{hour:'13',revenue:2450000},
        {hour:'14',revenue:1950000},{hour:'15',revenue:1350000},{hour:'16',revenue:870000},
      ];
      const demoRevenueChart = [
        {date:'Sen',revenue:9200000},{date:'Sel',revenue:11400000},{date:'Rab',revenue:8700000},
        {date:'Kam',revenue:13100000},{date:'Jum',revenue:15600000},
        {date:'Sab',revenue:18900000},{date:'Min',revenue:14200000},
      ];

      const apiSalesChart = Array.isArray(statsData.sales_chart) && statsData.sales_chart.length > 0
        ? statsData.sales_chart : demoSalesChart;
      const apiRevenueChart = Array.isArray(statsData.revenue_chart) && statsData.revenue_chart.length > 0
        ? statsData.revenue_chart : demoRevenueChart;

      stats = {
        sales_today:      statsData.sales_today      ?? 0,
        visitors_today:   statsData.visitors_today   ?? 0,
        tickets_sold:     statsData.tickets_sold     ?? 0,
        top_product:      statsData.top_product      ?? '-',
        revenue_today:    statsData.revenue_today    ?? 0,
        printer_status:   'disconnected',
        bluetooth_status: 'disconnected',
        scan_status:      'online',
        investor_revenue: statsData.investor_revenue ?? 0,
        peak_hour:        statsData.peak_hour        ?? 'N/A',
        sales_chart:      apiSalesChart,
        revenue_chart:    apiRevenueChart,
      };
    } catch (e: any) {
      console.error('loadData error:', e);
      debugError = 'loadData error: ' + (e.stack || e.message);
      // Fallback to full demo data if API fails
      stats = {
        sales_today: 0, visitors_today: 0, tickets_sold: 0,
        top_product: '-', revenue_today: 0,
        printer_status: 'disconnected', bluetooth_status: 'disconnected',
        scan_status: 'online', investor_revenue: 0, peak_hour: 'N/A',
        sales_chart: [
          {hour:'08',revenue:450000},{hour:'09',revenue:980000},{hour:'10',revenue:1200000},
          {hour:'11',revenue:1800000},{hour:'12',revenue:2100000},{hour:'13',revenue:2450000},
          {hour:'14',revenue:1950000},{hour:'15',revenue:1350000},{hour:'16',revenue:870000},
        ],
        revenue_chart: [
          {date:'Sen',revenue:9200000},{date:'Sel',revenue:11400000},{date:'Rab',revenue:8700000},
          {date:'Kam',revenue:13100000},{date:'Jum',revenue:15600000},
          {date:'Sab',revenue:18900000},{date:'Min',revenue:14200000},
        ],
      };
      recentTx = [];
    } finally {
      loading = false;
    }
    // Defer initialization to ensure Svelte has mounted the canvas element
    setTimeout(initChart, 100);
  }

  onMount(async () => {
    try {
      const chartModule = await import('chart.js/auto');
      ChartClass = chartModule.default || chartModule.Chart || chartModule;
    } catch (e: any) {
      console.error('Gagal memuat Chart.js secara dinamis:', e);
      debugError = 'onMount import error: ' + (e.stack || e.message);
    }
    await loadData();
  });

  function initChart() {
    try {
      if (!chartCanvas) {
        console.warn('initChart skipped: canvas element not ready');
        return;
      }
      if (!stats || !stats.sales_chart?.length) {
        console.warn('initChart skipped: stats data not ready or empty');
        return;
      }
      if (!ChartClass) {
        console.warn('initChart skipped: ChartClass is not loaded yet');
        debugError = 'initChart error: ChartClass is not loaded yet';
        return;
      }

      if (chart) {
        chart.destroy();
        chart = null;
      }

      const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
      const gridColor = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      const textColor = isDark ? '#9898a6' : '#6e6e80';

      chart = new ChartClass(chartCanvas, {
        type: 'bar',
        data: {
          labels: stats.sales_chart.map((d: any) => (d.hour ?? d.date ?? '') + (d.hour ? ':00' : '')),
          datasets: [{
            label: 'Pendapatan',
            data: stats.sales_chart.map((d: any) => d.revenue ?? 0),
            backgroundColor: 'rgba(99,102,241,0.7)',
            borderRadius: 6,
            borderSkipped: false
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { display: false },
            tooltip: { callbacks: { label: (ctx: any) => currency(ctx.parsed.y ?? 0) } }
          },
          scales: {
            x: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 11 } } },
            y: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 11 }, callback: (v: any) => 'Rp' + Number(v)/1000 + 'k' } }
          }
        }
      });
    } catch (e: any) {
      console.error('initChart error:', e);
      debugError = 'initChart error: ' + (e.stack || e.message);
    }
  }

  const statusMap: Record<string,{label:string;variant:any}> = {
    paid:      { label:'Lunas',   variant:'success' },
    pending:   { label:'Pending', variant:'warning' },
    refunded:  { label:'Refund',  variant:'danger'  },
    cancelled: { label:'Batal',   variant:'neutral' },
  };

  const quickActions = [
    { label:'Transaksi Baru', icon: Plus,    href:'/kasir',       color:'var(--brand-500)' },
    { label:'Scan Tiket',    icon: Scan,    href:'/tiket',       color:'var(--emerald)' },
    { label:'Print Ulang',   icon: Printer, href:'/kasir',       color:'var(--amber)' },
    { label:'Tambah Produk', icon: Plus,    href:'/produk',      color:'var(--violet)' },
  ];
</script>

<svelte:head><title>Dashboard — TiketKu</title></svelte:head>

<div class="dashboard">
  {#if debugError}
    <div style="background:#fee2e2; border:1px solid #fca5a5; color:#991b1b; padding:16px; margin-bottom:20px; border-radius:8px; font-family:monospace; white-space:pre-wrap; font-size: 0.8125rem; text-align: left;">
      <strong>Debug Error:</strong> {debugError}
    </div>
  {/if}
  <div class="page-header">
    <div>
      <h1>Dashboard</h1>
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">Ringkasan operasional hari ini</p>
    </div>
    <div style="display:flex;gap:8px;align-items:center;">
      <div class="system-status">
        <span class="dot {$btState === 'connected' ? 'dot-success' : 'dot-danger'} dot-pulse"></span>
        <span style="font-size:0.8125rem;color:var(--text-2);">
          Printer {$btState === 'connected' ? 'Online' : 'Offline'}
        </span>
      </div>
      <button class="btn btn-secondary btn-sm" on:click={loadData} disabled={loading}>
        <RefreshCw size={13} />
        Refresh
      </button>
    </div>
  </div>

  <div class="stats-grid fade-in">
    <StatCard label="Pendapatan Hari Ini" value={loading ? '...' : currency(stats?.revenue_today ?? 0)} sub="Total semua metode" trend={8.4} color="var(--brand-500)" {loading} />
    <StatCard label="Tiket Terjual"        value={loading ? '...' : number(stats?.tickets_sold ?? 0)} sub="Hari ini" trend={12.1} color="var(--emerald)" {loading} />
    <StatCard label="Total Penjualan"      value={loading ? '...' : currency(stats?.sales_today ?? 0)} sub="Setelah diskon" trend={-2.3} color="var(--amber)" {loading} />
    <StatCard label="Revenue Investor"     value={loading ? '...' : currency(stats?.investor_revenue ?? 0)} sub="Share produk" trend={5.7} color="var(--violet)" {loading} />
  </div>

  <div class="quick-actions fade-in-1">
    {#each quickActions as action}
      <a href={action.href} class="quick-action-btn card">
        <span class="qa-icon" style="background:{action.color}20;color:{action.color};">
          <svelte:component this={action.icon} size={18} />
        </span>
        <span class="qa-label">{action.label}</span>
        <ArrowUpRight size={14} style="color:var(--text-3)" />
      </a>
    {/each}
  </div>

  <div class="content-grid">
    <div class="card chart-card fade-in-2">
      <div class="card-header">
        <div>
          <h3>Penjualan per Jam</h3>
          <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">Hari ini &mdash; Peak: {stats?.peak_hour ?? '--'}</p>
        </div>
        <Activity size={18} style="color:var(--text-3)" />
      </div>
      <div class="chart-wrap">
        {#if loading}
          <div class="skel" style="width:100%;height:100%;"></div>
        {:else if !stats?.sales_chart?.length}
          <div style="height:100%;display:flex;align-items:center;justify-content:center;color:var(--text-3);font-size:0.875rem;">Belum ada data hari ini</div>
        {:else}
          <canvas bind:this={chartCanvas}></canvas>
        {/if}
      </div>
    </div>

    <div class="card fade-in-3">
      <div class="card-header">
        <div>
          <h3>Transaksi Terbaru</h3>
          <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">5 transaksi terakhir</p>
        </div>
        <a href="/laporan" class="btn btn-ghost btn-sm" style="gap:4px;">
          Lihat semua <ArrowUpRight size={13} />
        </a>
      </div>
      {#if loading}
        <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
          {#each Array(4) as _}<div class="skel" style="height:44px;"></div>{/each}
        </div>
      {:else if recentTx.length === 0}
        <div style="padding:32px;text-align:center;color:var(--text-3);font-size:0.875rem;">Belum ada transaksi</div>
      {:else}
        <div class="table-wrap" style="border:none;border-radius:0;box-shadow:none;">
          <table class="table">
            <thead>
              <tr><th>Invoice</th><th>Total</th><th>Status</th><th>Waktu</th></tr>
            </thead>
            <tbody>
              {#each recentTx as tx}
                <tr>
                  <td><span class="mono" style="font-size:0.8125rem;">{tx.invoice}</span></td>
                  <td style="font-weight:600;">{currency(tx.total)}</td>
                  <td><Badge variant={statusMap[tx.status]?.variant ?? 'neutral'}>{statusMap[tx.status]?.label ?? tx.status}</Badge></td>
                  <td style="color:var(--text-2);font-size:0.8125rem;">{relativeTime(tx.created_at)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>

  <div class="bottom-grid fade-in-4">
    <div class="card">
      <div class="card-header"><h3>Produk Terlaris</h3><TrendingUp size={16} style="color:var(--text-3);" /></div>
      {#if loading}
        <div style="padding:16px;"><div class="skel" style="height:60px;"></div></div>
      {:else}
        <div class="top-product">
          <div class="tp-icon"><Ticket size={24} style="color:var(--brand-500)" /></div>
          <div>
            <p style="font-weight:600;">{stats?.top_product ?? '-'}</p>
            <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{number(stats?.tickets_sold ?? 0)} tiket terjual hari ini</p>
          </div>
        </div>
      {/if}
    </div>

    <div class="card">
      <div class="card-header"><h3>Status Sistem</h3><span class="dot dot-success dot-pulse"></span></div>
      <div class="status-list">
        {#each [
          { label:'Printer Bluetooth', value: $btState === 'connected' ? 'Terhubung' : 'Terputus', ok: $btState === 'connected' },
          { label:'Scan QR',           value: (stats?.scan_status ?? 'offline') === 'online' ? 'Aktif' : 'Nonaktif', ok: (stats?.scan_status ?? 'offline') === 'online' },
          { label:'Database',          value: 'Terhubung', ok: true },
          { label:'Cache (Redis)',      value: 'Aktif',     ok: true },
        ] as s}
          <div class="status-row">
            <span class="dot {s.ok ? 'dot-success' : 'dot-danger'}"></span>
            <span style="flex:1;font-size:0.875rem;">{s.label}</span>
            <span style="font-size:0.8125rem;color:{s.ok ? 'var(--emerald)' : 'var(--rose)'};font-weight:500;">{s.value}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="card">
      <div class="card-header"><h3>Pendapatan Mingguan</h3><ArrowUpRight size={16} style="color:var(--emerald);" /></div>
      <div class="weekly-bars">
        {#if stats}
          {#each stats.revenue_chart as d}
            {@const max = Math.max(...stats.revenue_chart.map(r => r.revenue))}
            {@const h = max > 0 ? Math.max(4, (d.revenue / max) * 60) : 4}
            <div class="weekly-bar-wrap">
              <span class="weekly-amount">{(d.revenue/1000000).toFixed(1)}M</span>
              <div class="weekly-bar" style="height:{h}px"></div>
              <span class="weekly-day">{d.date}</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .dashboard { display:flex; flex-direction:column; gap:20px; max-width:1400px; }
  .page-header { display:flex; align-items:center; justify-content:space-between; flex-wrap:wrap; gap:12px; }
  .page-header h1 { font-size:1.5rem; font-weight:700; letter-spacing:-0.025em; }
  .system-status { display:flex; align-items:center; gap:6px; padding:6px 12px; background:var(--bg-surface); border:1px solid var(--border); border-radius:var(--r-full); }
  .stats-grid { display:grid; grid-template-columns:repeat(4,1fr); gap:14px; }
  .quick-actions { display:grid; grid-template-columns:repeat(4,1fr); gap:10px; }
  .quick-action-btn { display:flex; align-items:center; gap:10px; padding:14px 16px; cursor:pointer; transition:transform var(--ease-fast),box-shadow var(--ease-fast); text-decoration:none; color:var(--text-1); }
  .quick-action-btn:hover { transform:translateY(-2px); box-shadow:var(--shadow-md); }
  .qa-icon { width:36px; height:36px; border-radius:var(--r-md); display:flex; align-items:center; justify-content:center; flex-shrink:0; }
  .qa-label { flex:1; font-size:0.875rem; font-weight:500; }
  .content-grid { display:grid; grid-template-columns:1fr 380px; gap:14px; }
  .chart-card { overflow:hidden; }
  .card-header { display:flex; align-items:flex-start; justify-content:space-between; padding:18px 20px 14px; border-bottom:1px solid var(--border); }
  .card-header h3 { font-size:0.9375rem; }
  .chart-wrap { position: relative; height:240px; padding:16px; }
  .chart-wrap canvas { display: block; width:100%; height:100%; }
  .bottom-grid { display:grid; grid-template-columns:repeat(3,1fr); gap:14px; }
  .top-product { display:flex; align-items:center; gap:14px; padding:16px 20px; }
  .tp-icon { width:48px; height:48px; border-radius:var(--r-lg); background:var(--brand-50); display:flex; align-items:center; justify-content:center; flex-shrink:0; }
  [data-theme='dark'] .tp-icon { background:rgba(99,102,241,0.12); }
  .status-list { padding:12px 20px 16px; display:flex; flex-direction:column; gap:10px; }
  .status-row { display:flex; align-items:center; gap:10px; }
  .weekly-bars { display:flex; align-items:flex-end; justify-content:space-between; gap:6px; padding:16px 20px; height:110px; }
  .weekly-bar-wrap { display:flex; flex-direction:column; align-items:center; gap:4px; flex:1; }
  .weekly-amount { font-size:0.625rem; color:var(--text-3); white-space:nowrap; }
  .weekly-bar { width:100%; background:var(--brand-500); border-radius:var(--r-sm) var(--r-sm) 0 0; opacity:0.75; min-height:4px; }
  .weekly-day { font-size:0.6875rem; color:var(--text-3); }
  @media (max-width:1100px) { .content-grid { grid-template-columns:1fr; } .bottom-grid { grid-template-columns:repeat(2,1fr); } .stats-grid { grid-template-columns:repeat(2,1fr); } .quick-actions { grid-template-columns:repeat(2,1fr); } }
  @media (max-width:640px) {
    .stats-grid,.bottom-grid { grid-template-columns:1fr; }
    .quick-actions { grid-template-columns:repeat(2,1fr); gap:8px; }
    .quick-action-btn { padding:12px 10px; gap:8px; }
    .qa-label { font-size:0.8rem; }
  }
</style>
