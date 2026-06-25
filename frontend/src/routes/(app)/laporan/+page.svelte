<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  import { onMount, afterUpdate } from 'svelte';
  import { currency, number, date } from '$lib/utils/format';
  import { reportsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { SalesReport } from '$lib/types';
  import { BarChart3, Download, Calendar, Filter, RefreshCw } from 'lucide-svelte';

  let loading = true;
  let dateFrom = new Date(Date.now() - 7 * 86400000).toISOString().slice(0,10);
  let dateTo   = new Date().toISOString().slice(0,10);
  let salesData: SalesReport[] = [];
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let chartInitialized = false;

  $: totalRevenue = salesData.reduce((s,d) => s + (d.net_revenue ?? 0), 0);
  $: totalTx      = salesData.reduce((s,d) => s + (d.total_transactions ?? 0), 0);
  $: totalRefunds = salesData.reduce((s,d) => s + (d.total_refunds ?? 0), 0);
  $: avgPerDay    = salesData.length > 0 ? Math.round(totalRevenue / salesData.length) : 0;

  async function loadData() {
    loading = true;
    chartInitialized = false;
    try {
      const res = await reportsApi.sales({ from: dateFrom, to: dateTo });
      const d = (res as any)?.data;
      salesData = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat laporan', e.message);
      salesData = [];
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

  afterUpdate(() => {
    if (!loading && chartCanvas && salesData.length > 0 && !chartInitialized) {
      chartInitialized = true;
      initChart();
    }
  });

  function initChart() {
    if (!chartCanvas || salesData.length === 0) return;
    import('chart.js/auto').then(({ default: Chart }) => {
      if (chart) chart.destroy();
      const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
      const gridColor = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      const textColor = isDark ? '#9898a6' : '#6e6e80';
      chart = new Chart(chartCanvas, {
        type: 'line',
        data: {
          labels: salesData.map(d => d.date ? date(d.date, { day:'2-digit', month:'short' }) : ''),
          datasets: [
            { label:'Net Revenue', data: salesData.map(d => d.net_revenue ?? 0),
              borderColor:'rgba(99,102,241,0.9)', backgroundColor:'rgba(99,102,241,0.08)',
              borderWidth:2.5, pointRadius:4, fill:true, tension:0.4 },
            { label:'Refund', data: salesData.map(d => d.total_refunds ?? 0),
              borderColor:'rgba(244,63,94,0.7)', backgroundColor:'transparent',
              borderWidth:1.5, pointRadius:3, borderDash:[4,4], tension:0.4 },
          ]
        },
        options: {
          responsive:true, maintainAspectRatio:false,
          plugins: {
            legend: { position:'top', labels: { color:textColor, boxWidth:12, font:{size:12} } },
            tooltip: { callbacks: { label: (ctx: any) => currency(ctx.parsed.y ?? 0) } }
          },
          scales: {
            x: { grid:{color:gridColor}, ticks:{color:textColor,font:{size:11}} },
            y: { grid:{color:gridColor}, ticks:{color:textColor,font:{size:11},callback:(v:any)=>'Rp'+Number(v)/1000000+'jt'} }
          }
        }
      });
    });
  }
</script>

<svelte:head><title>Laporan Penjualan — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Penjualan Harian</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Analitik pendapatan per hari</p>
    </div>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" style="gap:6px;"><Download size={14} /> Export Excel</button>
      <button class="btn btn-secondary" style="gap:6px;"><Download size={14} /> Export PDF</button>
    </div>
  </div>

  <!-- Filter -->
  <div class="card" style="padding:14px 18px;">
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
      <Calendar size={16} style="color:var(--text-3);" />
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;white-space:nowrap;" for="date-from">Dari</label>
        <input id="date-from" class="input input-sm" type="date" bind:value={dateFrom} style="width:150px;" />
      </div>
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;white-space:nowrap;" for="date-to">Sampai</label>
        <input id="date-to" class="input input-sm" type="date" bind:value={dateTo} style="width:150px;" />
      </div>
      <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={loadData} disabled={loading}>
        <Filter size={13} /> {loading ? 'Memuat...' : 'Terapkan'}
      </button>
    </div>
  </div>

  <!-- Stats -->
  <div class="stats-row">
    {#each [
      { label:'Total Transaksi',   value: number(totalTx),        color:'var(--brand-500)' },
      { label:'Pendapatan Bersih', value: currency(totalRevenue), color:'var(--emerald)' },
      { label:'Total Refund',      value: currency(totalRefunds), color:'var(--rose)' },
      { label:'Rata-rata/Hari',    value: currency(avgPerDay),    color:'var(--amber)' },
    ] as s}
      <div class="stat-mini card">
        <span class="stat-lbl">{s.label}</span>
        <span class="stat-val" style="color:{s.color}">{loading ? '...' : s.value}</span>
      </div>
    {/each}
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">{#each Array(5) as _}<div class="skel" style="height:48px;"></div>{/each}</div>
  {:else}
    <!-- Chart -->
    <div class="card" style="padding:20px;">
      <h3 style="margin-bottom:16px;">Grafik Pendapatan</h3>
      <div style="height:280px;">
        {#if salesData.length === 0}
          <div style="height:100%;display:flex;align-items:center;justify-content:center;color:var(--text-3);font-size:0.875rem;">Tidak ada data untuk periode ini</div>
        {:else}
          <canvas bind:this={chartCanvas}></canvas>
        {/if}
      </div>
    </div>

    <!-- Table -->
    {#if salesData.length > 0}
      <div class="table-wrap">
        <table class="table">
          <thead><tr><th>Tanggal</th><th>Transaksi</th><th>Gross Revenue</th><th>Refund</th><th>Net Revenue</th></tr></thead>
          <tbody>
            {#each salesData as row}
              <tr>
                <td>{row.date ? date(row.date) : '-'}</td>
                <td>{number(row.total_transactions ?? 0)}</td>
                <td style="font-weight:500;">{currency(row.total_revenue ?? 0)}</td>
                <td style="color:var(--rose);">{(row.total_refunds ?? 0) > 0 ? '- ' + currency(row.total_refunds ?? 0) : '—'}</td>
                <td style="font-weight:700;color:var(--emerald);">{currency(row.net_revenue ?? 0)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">Tidak ada data untuk periode ini</div>
    {/if}
  {/if}
</div>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1200px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .stats-row { display:grid;grid-template-columns:repeat(4,1fr);gap:14px; }
  .stat-mini { padding:18px 20px; }
  .stat-lbl { font-size:0.75rem;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:var(--text-2);display:block;margin-bottom:8px; }
  .stat-val { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em;display:block; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
  @media (max-width:900px) { .stats-row { grid-template-columns:repeat(2,1fr); } }
  @media (max-width:480px) { .stats-row { grid-template-columns:1fr; } }
</style>
