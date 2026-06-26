<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  import { onMount, afterUpdate } from 'svelte';
  import { currency, number, date } from '$lib/utils/format';
  import { reportsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { SalesReport } from '$lib/types';
  import { BarChart3, Download, Calendar, Filter, RefreshCw } from 'lucide-svelte';
  import { exportToExcel, exportToPDF } from '$lib/utils/export';

  let loading = true;
  let dateFrom = new Date(Date.now() - 30 * 86400000).toISOString().slice(0,10); // default to 30 days so monthly/yearly have visual data
  let dateTo   = new Date().toISOString().slice(0,10);
  let salesData: SalesReport[] = [];
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let chartInitialized = false;
  let viewMode: 'daily' | 'monthly' | 'yearly' = 'daily';

  $: aggregatedData = aggregateReport(salesData, viewMode);
  $: totalRevenue = aggregatedData.reduce((s,d) => s + (d.net_revenue ?? 0), 0);
  $: totalTx      = aggregatedData.reduce((s,d) => s + (d.total_transactions ?? 0), 0);
  $: totalRefunds = aggregatedData.reduce((s,d) => s + (d.total_refunds ?? 0), 0);
  $: avgPerPeriod = aggregatedData.length > 0 ? Math.round(totalRevenue / aggregatedData.length) : 0;

  function aggregateReport(data: SalesReport[], mode: 'daily' | 'monthly' | 'yearly') {
    if (mode === 'daily') return data;

    const groups: Record<string, {
      date: string;
      total_transactions: number;
      total_revenue: number;
      total_refunds: number;
      net_revenue: number;
    }> = {};

    data.forEach(row => {
      if (!row.date) return;
      let key = '';
      if (mode === 'monthly') {
        key = row.date.slice(0, 7); // YYYY-MM
      } else {
        key = row.date.slice(0, 4); // YYYY
      }

      if (!groups[key]) {
        groups[key] = {
          date: key,
          total_transactions: 0,
          total_revenue: 0,
          total_refunds: 0,
          net_revenue: 0
        };
      }

      groups[key].total_transactions += row.total_transactions ?? 0;
      groups[key].total_revenue += row.total_revenue ?? 0;
      groups[key].total_refunds += row.total_refunds ?? 0;
      groups[key].net_revenue += row.net_revenue ?? 0;
    });

    return Object.values(groups).sort((a, b) => a.date.localeCompare(b.date));
  }

  function setViewMode(mode: 'daily' | 'monthly' | 'yearly') {
    viewMode = mode;
    chartInitialized = false;
  }

  function formatRowDate(dateStr: string | undefined) {
    if (!dateStr) return '-';
    if (viewMode === 'daily') {
      return date(dateStr);
    } else if (viewMode === 'monthly') {
      return date(new Date(dateStr + '-02'), { year: 'numeric', month: 'long' });
    } else {
      return dateStr;
    }
  }

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
    if (!loading && chartCanvas && aggregatedData.length > 0 && !chartInitialized) {
      chartInitialized = true;
      initChart();
    }
  });

  function initChart() {
    if (!chartCanvas || aggregatedData.length === 0) return;
    import('chart.js/auto').then(({ default: Chart }) => {
      if (chart) chart.destroy();
      const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
      const gridColor = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      const textColor = isDark ? '#9898a6' : '#6e6e80';

      const labels = aggregatedData.map(d => {
        if (!d.date) return '';
        if (viewMode === 'daily') {
          return date(d.date, { day:'2-digit', month:'short' });
        } else if (viewMode === 'monthly') {
          return date(new Date(d.date + '-02'), { month:'short', year:'numeric' });
        } else {
          return d.date;
        }
      });

      chart = new Chart(chartCanvas, {
        type: 'line',
        data: {
          labels,
          datasets: [
            { label:'Net Revenue', data: aggregatedData.map(d => d.net_revenue ?? 0),
              borderColor:'rgba(99,102,241,0.9)', backgroundColor:'rgba(99,102,241,0.08)',
              borderWidth:2.5, pointRadius:4, fill:true, tension:0.4 },
            { label:'Refund', data: aggregatedData.map(d => d.total_refunds ?? 0),
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

  function handleExportExcel() {
    const dateCol = viewMode === 'daily' ? 'Tanggal' : viewMode === 'monthly' ? 'Bulan' : 'Tahun';
    const headers = [dateCol, 'Transaksi', 'Gross Revenue', 'Refund', 'Net Revenue'];
    const rows = aggregatedData.map(row => [
      formatRowDate(row.date),
      row.total_transactions ?? 0,
      row.total_revenue ?? 0,
      row.total_refunds ?? 0,
      row.net_revenue ?? 0
    ]);
    exportToExcel(headers, rows, `laporan_penjualan_${viewMode}_${dateFrom}_to_${dateTo}`);
  }

  function handleExportPDF() {
    const dateCol = viewMode === 'daily' ? 'Tanggal' : viewMode === 'monthly' ? 'Bulan' : 'Tahun';
    const headers = [dateCol, 'Transaksi', 'Gross Revenue', 'Refund', 'Net Revenue'];
    const rows = aggregatedData.map(row => [
      formatRowDate(row.date),
      number(row.total_transactions ?? 0),
      currency(row.total_revenue ?? 0),
      (row.total_refunds ?? 0) > 0 ? '- ' + currency(row.total_refunds ?? 0) : '—',
      currency(row.net_revenue ?? 0)
    ]);
    const stats = [
      { label: 'Total Transaksi', value: number(totalTx) },
      { label: 'Pendapatan Bersih', value: currency(totalRevenue) },
      { label: 'Total Refund', value: currency(totalRefunds) },
      { label: viewMode === 'daily' ? 'Rata-rata/Hari' : viewMode === 'monthly' ? 'Rata-rata/Bulan' : 'Rata-rata/Tahun', value: currency(avgPerPeriod) }
    ];
    const reportTitle = viewMode === 'daily' ? 'Laporan Penjualan Harian' : viewMode === 'monthly' ? 'Laporan Penjualan Bulanan' : 'Laporan Penjualan Tahunan';
    exportToPDF(reportTitle, `${date(dateFrom)} - ${date(dateTo)}`, stats, headers, rows);
  }
</script>

<svelte:head><title>Laporan Penjualan — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Laporan Penjualan</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Analisis dan visualisasi data pendapatan</p>
    </div>
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
      <div class="tabs">
        <button class="tab" class:active={viewMode === 'daily'} on:click={() => setViewMode('daily')}>Harian</button>
        <button class="tab" class:active={viewMode === 'monthly'} on:click={() => setViewMode('monthly')}>Bulanan</button>
        <button class="tab" class:active={viewMode === 'yearly'} on:click={() => setViewMode('yearly')}>Tahunan</button>
      </div>
      <div style="display:flex;gap:8px;">
        <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportExcel}><Download size={14} /> Export Excel</button>
        <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportPDF}><Download size={14} /> Export PDF</button>
      </div>
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
      { label: viewMode === 'daily' ? 'Rata-rata/Hari' : viewMode === 'monthly' ? 'Rata-rata/Bulan' : 'Rata-rata/Tahun', value: currency(avgPerPeriod), color:'var(--amber)' },
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
      <h3 style="margin-bottom:16px;">Grafik Pendapatan ({viewMode === 'daily' ? 'Harian' : viewMode === 'monthly' ? 'Bulanan' : 'Tahunan'})</h3>
      <div style="height:280px;">
        {#if aggregatedData.length === 0}
          <div style="height:100%;display:flex;align-items:center;justify-content:center;color:var(--text-3);font-size:0.875rem;">Tidak ada data untuk periode ini</div>
        {:else}
          <canvas bind:this={chartCanvas}></canvas>
        {/if}
      </div>
    </div>

    <!-- Table -->
    {#if aggregatedData.length > 0}
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th>{viewMode === 'daily' ? 'Tanggal' : viewMode === 'monthly' ? 'Bulan' : 'Tahun'}</th>
              <th>Transaksi</th>
              <th>Gross Revenue</th>
              <th>Refund</th>
              <th>Net Revenue</th>
            </tr>
          </thead>
          <tbody>
            {#each aggregatedData as row}
              <tr>
                <td>{formatRowDate(row.date)}</td>
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
  .page { display:flex;flex-direction:column;gap:20px;max-width:1400px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .stats-row { display:grid;grid-template-columns:repeat(4,1fr);gap:14px; }
  .stat-mini { padding:18px 20px; }
  .stat-lbl { font-size:0.75rem;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:var(--text-2);display:block;margin-bottom:8px; }
  .stat-val { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em;display:block; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }

  /* Tabs switcher styling */
  .tabs {
    display:flex;gap:4px;
    background:var(--bg-subtle);
    border:1px solid var(--border);
    border-radius:var(--r-lg);
    padding:4px;
  }
  .tab {
    display:flex;align-items:center;justify-content:center;
    padding:7px 16px;
    border-radius:var(--r-md);
    font-size:0.8125rem;font-weight:500;
    color:var(--text-2);
    transition:all var(--ease-fast);
    border:none;
    background:none;
    cursor:pointer;
    font-family:var(--font-sans);
  }
  .tab:hover { color:var(--text-1); }
  .tab.active {
    background:var(--bg-elevated);
    color:var(--text-1);
    box-shadow:var(--shadow-sm);
    font-weight:600;
  }

  @media (max-width:900px) { .stats-row { grid-template-columns:repeat(2,1fr); } }
  @media (max-width:480px) { .stats-row { grid-template-columns:1fr; } }
</style>
