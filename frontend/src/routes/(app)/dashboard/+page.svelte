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
  let filterRange = 'today';
  let customFrom = '';
  let customTo = '';

  function getDateParams() {
    const params: Record<string, string> = {};
    const tzOffset = new Date().getTimezoneOffset() * 60000;
    const getLocalDateString = (d: Date) => new Date(d.getTime() - tzOffset).toISOString().split('T')[0];

    const today = new Date();
    
    if (filterRange === 'today') {
      params.from = getLocalDateString(today);
      params.to = getLocalDateString(today);
    } else if (filterRange === 'yesterday') {
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      params.from = getLocalDateString(yesterday);
      params.to = getLocalDateString(yesterday);
    } else if (filterRange === 'last7') {
      const start = new Date();
      start.setDate(start.getDate() - 6);
      params.from = getLocalDateString(start);
      params.to = getLocalDateString(today);
    } else if (filterRange === 'last30') {
      const start = new Date();
      start.setDate(start.getDate() - 29);
      params.from = getLocalDateString(start);
      params.to = getLocalDateString(today);
    } else if (filterRange === 'thisMonth') {
      const start = new Date(today.getFullYear(), today.getMonth(), 1);
      params.from = getLocalDateString(start);
      params.to = getLocalDateString(today);
    } else if (filterRange === 'thisYear') {
      const start = new Date(today.getFullYear(), 0, 1);
      params.from = getLocalDateString(start);
      params.to = getLocalDateString(today);
    } else if (filterRange === 'custom') {
      if (customFrom) params.from = customFrom;
      if (customTo) params.to = customTo;
    }
    return params;
  }

  async function loadData() {
    loading = true;
    try {
      const dateParams = getDateParams();
      const [statsRes, txRes] = await Promise.all([
        dashboardApi.stats(dateParams),
        transactionsApi.list({ per_page: '5', ...dateParams }),
      ]);
      const statsData = (statsRes as any)?.data ?? {};
      const txData    = (txRes as any)?.data;
      recentTx = Array.isArray(txData) ? txData : [];

      let apiSalesChart: any[] = [];
      const monthNamesShort = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agt', 'Sep', 'Okt', 'Nov', 'Des'];

      if (Array.isArray(statsData.sales_chart) && statsData.sales_chart.length > 0) {
        if (filterRange === 'thisYear') {
          // Aggregate daily data by month index
          const monthlyGroups: Record<number, number> = {};
          for (let m = 0; m < 12; m++) {
            monthlyGroups[m] = 0;
          }
          statsData.sales_chart.forEach((item: any) => {
            if (!item.date) return;
            const parts = item.date.split('/');
            if (parts.length === 2) {
              const monthIdx = parseInt(parts[1], 10) - 1;
              if (monthIdx >= 0 && monthIdx < 12) {
                monthlyGroups[monthIdx] += item.revenue ?? 0;
              }
            }
          });
          apiSalesChart = monthNamesShort.map((name, idx) => ({
            date: name,
            revenue: monthlyGroups[idx]
          }));
        } else {
          apiSalesChart = statsData.sales_chart;
        }
      } else {
        // Fallback to real-time zeroed chart data representation instead of placeholders
        if (filterRange === 'today' || filterRange === 'yesterday') {
          const hours = ['08', '09', '10', '11', '12', '13', '14', '15', '16', '17', '18'];
          apiSalesChart = hours.map(h => ({ hour: h, revenue: 0 }));
        } else if (filterRange === 'thisYear') {
          apiSalesChart = monthNamesShort.map(name => ({ date: name, revenue: 0 }));
        } else {
          const params = getDateParams();
          const start = params.from ? new Date(params.from) : new Date();
          const end = params.to ? new Date(params.to) : new Date();
          const dateList = [];
          for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
            const label = String(d.getDate()).padStart(2, '0') + '/' + String(d.getMonth() + 1).padStart(2, '0');
            dateList.push({ date: label, revenue: 0 });
          }
          apiSalesChart = dateList;
        }
      }

      let apiRevenueChart: any[] = [];
      if (Array.isArray(statsData.revenue_chart) && statsData.revenue_chart.length > 0) {
        apiRevenueChart = statsData.revenue_chart;
      } else {
        const days = ['Sen', 'Sel', 'Rab', 'Kam', 'Jum', 'Sab', 'Min'];
        apiRevenueChart = days.map(d => ({ date: d, revenue: 0 }));
      }

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
      // Zeroed-out fallback stats
      const monthNamesShort = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agt', 'Sep', 'Okt', 'Nov', 'Des'];
      let apiSalesChart = [];
      if (filterRange === 'today' || filterRange === 'yesterday') {
        const hours = ['08', '09', '10', '11', '12', '13', '14', '15', '16', '17', '18'];
        apiSalesChart = hours.map(h => ({ hour: h, revenue: 0 }));
      } else if (filterRange === 'thisYear') {
        apiSalesChart = monthNamesShort.map(name => ({ date: name, revenue: 0 }));
      } else {
        const params = getDateParams();
        const start = params.from ? new Date(params.from) : new Date();
        const end = params.to ? new Date(params.to) : new Date();
        const dateList = [];
        for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
          const label = String(d.getDate()).padStart(2, '0') + '/' + String(d.getMonth() + 1).padStart(2, '0');
          dateList.push({ date: label, revenue: 0 });
        }
        apiSalesChart = dateList;
      }

      stats = {
        sales_today: 0, visitors_today: 0, tickets_sold: 0,
        top_product: '-', revenue_today: 0,
        printer_status: 'disconnected', bluetooth_status: 'disconnected',
        scan_status: 'online', investor_revenue: 0, peak_hour: 'N/A',
        sales_chart: apiSalesChart,
        revenue_chart: ['Sen', 'Sel', 'Rab', 'Kam', 'Jum', 'Sab', 'Min'].map(d => ({ date: d, revenue: 0 })),
      };
      recentTx = [];
    } finally {
      loading = false;
    }
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
          labels: stats.sales_chart.map((d: any) => d.hour ? d.hour + ':00' : d.date ?? ''),
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

  // Dynamic titles mapping for stats cards
  $: revenueLabel = filterRange === 'today' ? 'Pendapatan Hari Ini'
                  : filterRange === 'yesterday' ? 'Pendapatan Kemarin'
                  : filterRange === 'last7' ? 'Pendapatan 7 Hari Terakhir'
                  : filterRange === 'last30' ? 'Pendapatan 30 Hari Terakhir'
                  : filterRange === 'thisMonth' ? 'Pendapatan Bulan Ini'
                  : filterRange === 'thisYear' ? 'Pendapatan Tahun Ini'
                  : 'Pendapatan Periode Terpilih';

  $: ticketsLabel = filterRange === 'today' ? 'Tiket Terjual'
                  : filterRange === 'yesterday' ? 'Tiket Terjual Kemarin'
                  : filterRange === 'last7' ? 'Tiket Terjual (7 Hari)'
                  : filterRange === 'last30' ? 'Tiket Terjual (30 Hari)'
                  : filterRange === 'thisMonth' ? 'Tiket Terjual Bulan Ini'
                  : filterRange === 'thisYear' ? 'Tiket Terjual Tahun Ini'
                  : 'Tiket Terjual';

  $: salesLabel = filterRange === 'today' ? 'Total Penjualan'
                : filterRange === 'yesterday' ? 'Total Penjualan Kemarin'
                : filterRange === 'last7' ? 'Total Penjualan (7 Hari)'
                : filterRange === 'last30' ? 'Total Penjualan (30 Hari)'
                : filterRange === 'thisMonth' ? 'Total Penjualan Bulan Ini'
                : filterRange === 'thisYear' ? 'Total Penjualan Tahun Ini'
                : 'Total Penjualan';

  $: investorLabel = filterRange === 'today' ? 'Revenue Investor'
                   : filterRange === 'yesterday' ? 'Revenue Investor Kemarin'
                   : filterRange === 'last7' ? 'Revenue Investor (7 Hari)'
                   : filterRange === 'last30' ? 'Revenue Investor (30 Hari)'
                   : filterRange === 'thisMonth' ? 'Revenue Investor Bulan Ini'
                   : filterRange === 'thisYear' ? 'Revenue Investor Tahun Ini'
                   : 'Revenue Investor';

  $: cardSub = filterRange === 'today' ? 'Hari ini'
             : filterRange === 'yesterday' ? 'Kemarin'
             : filterRange === 'last7' ? '7 hari terakhir'
             : filterRange === 'last30' ? '30 hari terakhir'
             : filterRange === 'thisMonth' ? 'Bulan ini'
             : filterRange === 'thisYear' ? 'Tahun ini'
             : 'Periode terpilih';

  $: chartTitle = filterRange === 'today' || filterRange === 'yesterday' ? 'Penjualan per Jam'
                : filterRange === 'thisYear' ? 'Penjualan per Bulan'
                : 'Penjualan Harian';

  $: totalRevenue = stats?.sales_chart?.reduce((s: number, d: any) => s + (d.revenue ?? 0), 0) ?? 0;

  $: chartSubtitle = filterRange === 'today' || filterRange === 'yesterday'
    ? `Hari ini — Peak: ${stats?.peak_hour ?? 'N/A'}`
    : `Periode ini — Total: ${currency(totalRevenue)}`;
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
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">
        {#if filterRange === 'today'}
          Ringkasan operasional hari ini
        {:else if filterRange === 'yesterday'}
          Ringkasan operasional kemarin
        {:else if filterRange === 'last7'}
          Ringkasan operasional 7 hari terakhir
        {:else if filterRange === 'last30'}
          Ringkasan operasional 30 hari terakhir
        {:else if filterRange === 'thisMonth'}
          Ringkasan operasional bulan ini
        {:else if filterRange === 'thisYear'}
          Ringkasan operasional tahun ini
        {:else}
          Ringkasan operasional kustom
        {/if}
      </p>
    </div>
    <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;">
      <select class="input input-sm" style="width:160px;padding:4px 8px;font-size:0.8125rem;" bind:value={filterRange} on:change={loadData}>
        <option value="today">Hari Ini (Harian)</option>
        <option value="yesterday">Kemarin</option>
        <option value="last7">7 Hari Terakhir</option>
        <option value="last30">30 Hari Terakhir</option>
        <option value="thisMonth">Bulan Ini (Bulanan)</option>
        <option value="thisYear">Tahun Ini (Tahunan)</option>
        <option value="custom">Kustom Tanggal...</option>
      </select>

      {#if filterRange === 'custom'}
        <div style="display:flex;gap:4px;align-items:center;">
          <input type="date" class="input input-sm" style="width:130px;padding:4px 8px;font-size:0.8125rem;" bind:value={customFrom} on:change={loadData} />
          <span style="font-size:0.8125rem;color:var(--text-3);">s/d</span>
          <input type="date" class="input input-sm" style="width:130px;padding:4px 8px;font-size:0.8125rem;" bind:value={customTo} on:change={loadData} />
        </div>
      {/if}

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
    <StatCard label={revenueLabel} value={loading ? '...' : currency(stats?.revenue_today ?? 0)} sub={cardSub} trend={8.4} color="var(--brand-500)" {loading} />
    <StatCard label={ticketsLabel}  value={loading ? '...' : number(stats?.tickets_sold ?? 0)} sub={cardSub} trend={12.1} color="var(--emerald)" {loading} />
    <StatCard label={salesLabel}    value={loading ? '...' : currency(stats?.sales_today ?? 0)} sub={cardSub} trend={-2.3} color="var(--amber)" {loading} />
    <StatCard label={investorLabel} value={loading ? '...' : currency(stats?.investor_revenue ?? 0)} sub={cardSub} trend={5.7} color="var(--violet)" {loading} />
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
          <h3>{chartTitle}</h3>
          <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{chartSubtitle}</p>
        </div>
        <Activity size={18} style="color:var(--text-3)" />
      </div>
      <div class="chart-wrap">
        {#if loading}
          <div class="skel" style="width:100%;height:100%;"></div>
        {:else if !stats?.sales_chart?.length}
          <div style="height:100%;display:flex;align-items:center;justify-content:center;color:var(--text-3);font-size:0.875rem;">Belum ada data untuk periode ini</div>
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
            <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{number(stats?.tickets_sold ?? 0)} tiket terjual pada periode ini</p>
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
