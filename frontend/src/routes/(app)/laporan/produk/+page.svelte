<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  import { onMount, afterUpdate } from 'svelte';
  import { currency, number, date } from '$lib/utils/format';
  import { reportsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { ProductReport } from '$lib/types';
  import { TrendingUp, TrendingDown, Download, Filter, Calendar } from 'lucide-svelte';
  import { exportToExcel, exportToPDF } from '$lib/utils/export';

  let loading = true;
  let dateFrom = new Date(Date.now() - 30 * 86400000).toISOString().slice(0,10);
  let dateTo   = new Date().toISOString().slice(0,10);
  let products: ProductReport[] = [];
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let chartInitialized = false;

  $: totalRevenue = products.reduce((s,p) => s + (p.revenue ?? 0), 0);
  $: totalQty     = products.reduce((s,p) => s + (p.qty_sold ?? 0), 0);

  async function loadData() {
    loading = true;
    chartInitialized = false;
    try {
      const res = await reportsApi.products({ from: dateFrom, to: dateTo });
      const d = (res as any)?.data;
      products = Array.isArray(d) ? d : [];
    } catch (e: any) {
      toast.error('Gagal memuat laporan produk', e.message);
      products = [];
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

  afterUpdate(() => {
    if (!loading && chartCanvas && products.length > 0 && !chartInitialized) {
      chartInitialized = true;
      initChart();
    }
  });

  function initChart() {
    if (!chartCanvas || products.length === 0) return;
    import('chart.js/auto').then(({ default: Chart }) => {
      if (chart) chart.destroy();
      const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
      const textColor = isDark ? '#9898a6' : '#6e6e80';
      const gridColor = isDark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      const top10 = products.slice(0, 10);
      chart = new Chart(chartCanvas, {
        type: 'bar',
        data: {
          labels: top10.map(p => p.product_name ?? '-'),
          datasets: [{
            label: 'Revenue',
            data: top10.map(p => p.revenue ?? 0),
            backgroundColor: 'rgba(99,102,241,0.75)',
            borderRadius: 6,
            borderSkipped: false,
          }]
        },
        options: {
          indexAxis: 'y',
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { display: false },
            tooltip: { callbacks: { label: (ctx: any) => currency(ctx.parsed.x ?? 0) } }
          },
          scales: {
            x: { grid:{color:gridColor}, ticks:{color:textColor,font:{size:11},callback:(v:any)=>'Rp'+Number(v)/1000000+'jt'} },
            y: { grid:{display:false}, ticks:{color:textColor,font:{size:11}} }
          }
        }
      });
    });
  }

  function handleExportExcel() {
    const headers = ['#', 'Produk', 'Kategori', 'Qty Terjual', 'Revenue', 'Growth'];
    const rows = products.map((p, i) => [
      i + 1,
      p.product_name ?? '-',
      p.category ?? '-',
      p.qty_sold ?? 0,
      p.revenue ?? 0,
      p.growth_pct !== undefined && p.growth_pct !== null ? `${p.growth_pct}%` : '-'
    ]);
    exportToExcel(headers, rows, `laporan_produk_${dateFrom}_to_${dateTo}`);
  }

  function handleExportPDF() {
    const headers = ['#', 'Produk', 'Kategori', 'Qty Terjual', 'Revenue', 'Growth'];
    const rows = products.map((p, i) => [
      i + 1,
      p.product_name ?? '-',
      p.category ?? '-',
      number(p.qty_sold ?? 0),
      currency(p.revenue ?? 0),
      p.growth_pct !== undefined && p.growth_pct !== null ? `${p.growth_pct}%` : '-'
    ]);
    const stats = [
      { label: 'Total Revenue', value: currency(totalRevenue) },
      { label: 'Total Qty Terjual', value: number(totalQty) },
      { label: 'Jumlah Produk', value: String(products.length) }
    ];
    exportToPDF('Laporan Penjualan Per Produk', `${date(dateFrom)} - ${date(dateTo)}`, stats, headers, rows);
  }
</script>

<svelte:head><title>Laporan Per Produk — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Laporan Per Produk</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">{products.length} produk</p>
    </div>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportExcel}><Download size={14} /> Export Excel</button>
      <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportPDF}><Download size={14} /> Export PDF</button>
    </div>
  </div>

  <!-- Filter -->
  <div class="card" style="padding:14px 18px;">
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
      <Calendar size={16} style="color:var(--text-3);" />
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="pf">Dari</label>
        <input id="pf" class="input input-sm" type="date" bind:value={dateFrom} style="width:150px;" />
      </div>
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="pt">Sampai</label>
        <input id="pt" class="input input-sm" type="date" bind:value={dateTo} style="width:150px;" />
      </div>
      <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={loadData} disabled={loading}>
        <Filter size={13} /> {loading ? 'Memuat...' : 'Terapkan'}
      </button>
    </div>
  </div>

  <div class="stats-row">
    <div class="stat-mini card"><span class="stat-lbl">Total Revenue</span><span class="stat-val" style="color:var(--emerald)">{loading ? '...' : currency(totalRevenue)}</span></div>
    <div class="stat-mini card"><span class="stat-lbl">Total Qty Terjual</span><span class="stat-val" style="color:var(--brand-500)">{loading ? '...' : number(totalQty)}</span></div>
    <div class="stat-mini card"><span class="stat-lbl">Jumlah Produk</span><span class="stat-val" style="color:var(--amber)">{products.length}</span></div>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">{#each Array(5) as _}<div class="skel" style="height:48px;"></div>{/each}</div>
  {:else if products.length === 0}
    <div class="empty-state">Tidak ada data produk</div>
  {:else}
    <!-- Chart -->
    <div class="card" style="padding:20px;">
      <h3 style="margin-bottom:16px;">Top 10 Produk by Revenue</h3>
      <div style="height:320px;"><canvas bind:this={chartCanvas}></canvas></div>
    </div>

    <!-- Table -->
    <div class="table-wrap">
      <table class="table">
        <thead><tr><th>#</th><th>Produk</th><th>Kategori</th><th>Qty Terjual</th><th>Revenue</th><th>Growth</th></tr></thead>
        <tbody>
          {#each products as p, i}
            <tr>
              <td style="color:var(--text-3);">{i+1}</td>
              <td style="font-weight:600;">{p.product_name ?? '-'}</td>
              <td style="color:var(--text-2);">{p.category ?? '-'}</td>
              <td>{number(p.qty_sold ?? 0)}</td>
              <td style="font-weight:500;">{currency(p.revenue ?? 0)}</td>
              <td>
                {#if p.growth_pct !== undefined && p.growth_pct !== null}
                  <span style="display:flex;align-items:center;gap:4px;font-weight:600;color:{(p.growth_pct ?? 0) >= 0 ? 'var(--emerald)' : 'var(--rose)'};">
                    {#if (p.growth_pct ?? 0) >= 0}<TrendingUp size={13} />{:else}<TrendingDown size={13} />{/if}
                    {(p.growth_pct ?? 0) > 0 ? '+' : ''}{p.growth_pct}%
                  </span>
                {:else}
                  <span style="color:var(--text-3);">-</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1400px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .stats-row { display:grid;grid-template-columns:repeat(3,1fr);gap:14px; }
  .stat-mini { padding:18px 20px; }
  .stat-lbl { font-size:0.75rem;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:var(--text-2);display:block;margin-bottom:8px; }
  .stat-val { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em;display:block; }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
  @media (max-width:640px) { .stats-row { grid-template-columns:1fr; } }
</style>
