<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  import { onMount, afterUpdate } from 'svelte';
  import { currency, date } from '$lib/utils/format';
  import { api } from '$lib/api/client';
  import { toast } from '$lib/stores/toast';
  import { CreditCard, Download, Calendar, Filter } from 'lucide-svelte';
  import { exportToExcel, exportToPDF } from '$lib/utils/export';

  let loading = true;
  let dateFrom = new Date(Date.now() - 30 * 86400000).toISOString().slice(0,10);
  let dateTo   = new Date().toISOString().slice(0,10);
  let reportData: any[] = [];
  let chartCanvas: HTMLCanvasElement;
  let chart: any;
  let chartInitialized = false;

  $: totalRevenue = reportData.reduce((s,d) => s + (d.total_amount ?? 0), 0);
  $: totalTx      = reportData.reduce((s,d) => s + (d.transaction_count ?? 0), 0);

  async function loadData() {
    loading = true;
    chartInitialized = false;
    try {
      // Use transactions grouped by payment method
      const res = await api.get<any[]>(`/transactions?from=${dateFrom}&to=${dateTo}&per_page=1000`);
      const txs = (res as any)?.data ?? [];

      // Group by payment method from payments
      const methodMap: Record<string, { method: string; total_amount: number; transaction_count: number; fee: number }> = {};
      for (const tx of txs) {
        // We'll aggregate from the transactions list
        const method = tx.payment_method ?? 'cash';
        if (!methodMap[method]) {
          methodMap[method] = { method, total_amount: 0, transaction_count: 0, fee: 0 };
        }
        methodMap[method].total_amount += tx.total ?? 0;
        methodMap[method].transaction_count += 1;
      }
      reportData = Object.values(methodMap).sort((a, b) => b.total_amount - a.total_amount);
    } catch (e: any) {
      toast.error('Gagal memuat laporan', e.message);
      reportData = [];
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

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
      const colors = ['rgba(99,102,241,0.8)','rgba(16,185,129,0.8)','rgba(245,158,11,0.8)','rgba(244,63,94,0.8)','rgba(139,92,246,0.8)'];
      chart = new Chart(chartCanvas, {
        type: 'doughnut',
        data: {
          labels: reportData.map(d => d.method),
          datasets: [{
            data: reportData.map(d => d.total_amount),
            backgroundColor: colors,
            borderWidth: 2,
            borderColor: isDark ? '#17171e' : '#fff',
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { position: 'right', labels: { color: textColor, font: { size: 12 } } },
            tooltip: { callbacks: { label: (ctx: any) => `${ctx.label}: ${currency(ctx.parsed ?? 0)}` } }
          }
        }
      });
    });
  }

  const methodNames: Record<string, string> = {
    cash: 'Tunai', qris: 'QRIS', transfer: 'Transfer',
    debit: 'Debit', ewallet: 'E-Wallet',
  };

  function handleExportExcel() {
    const headers = ['Metode Pembayaran', 'Transaksi', 'Total Pendapatan', 'Persentase'];
    const rows = reportData.map(row => {
      const pct = totalRevenue > 0 ? ((row.total_amount / totalRevenue) * 100).toFixed(1) : '0';
      return [
        methodNames[row.method] ?? row.method,
        row.transaction_count ?? 0,
        row.total_amount ?? 0,
        `${pct}%`
      ];
    });
    exportToExcel(headers, rows, `laporan_metode_bayar_${dateFrom}_to_${dateTo}`);
  }

  function handleExportPDF() {
    const headers = ['Metode Pembayaran', 'Transaksi', 'Total Pendapatan', 'Persentase'];
    const rows = reportData.map(row => {
      const pct = totalRevenue > 0 ? ((row.total_amount / totalRevenue) * 100).toFixed(1) : '0';
      return [
        methodNames[row.method] ?? row.method,
        row.transaction_count ?? 0,
        currency(row.total_amount ?? 0),
        `${pct}%`
      ];
    });
    const stats = [
      { label: 'Total Transaksi', value: String(totalTx) },
      { label: 'Total Revenue', value: currency(totalRevenue) },
      { label: 'Metode Digunakan', value: String(reportData.length) }
    ];
    exportToPDF('Laporan Metode Pembayaran', `${date(dateFrom)} - ${date(dateTo)}`, stats, headers, rows);
  }
</script>

<svelte:head><title>Laporan Metode Bayar — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Laporan Metode Pembayaran</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Distribusi metode bayar yang digunakan</p>
    </div>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportExcel}><Download size={14} /> Export Excel</button>
      <button class="btn btn-secondary" style="gap:6px;" on:click={handleExportPDF}><Download size={14} /> Export PDF</button>
    </div>
  </div>

  <div class="card" style="padding:14px 18px;">
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
      <Calendar size={16} style="color:var(--text-3);" />
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="mf">Dari</label>
        <input id="mf" class="input input-sm" type="date" bind:value={dateFrom} style="width:150px;" />
      </div>
      <div style="display:flex;align-items:center;gap:8px;">
        <label class="label" style="margin:0;" for="mt">Sampai</label>
        <input id="mt" class="input input-sm" type="date" bind:value={dateTo} style="width:150px;" />
      </div>
      <button class="btn btn-primary btn-sm" style="gap:5px;" on:click={loadData} disabled={loading}>
        <Filter size={13} /> {loading ? 'Memuat...' : 'Terapkan'}
      </button>
    </div>
  </div>

  <div class="stats-row">
    <div class="stat-mini card"><span class="stat-lbl">Total Transaksi</span><span class="stat-val" style="color:var(--brand-500)">{loading ? '...' : totalTx}</span></div>
    <div class="stat-mini card"><span class="stat-lbl">Total Revenue</span><span class="stat-val" style="color:var(--emerald)">{loading ? '...' : currency(totalRevenue)}</span></div>
    <div class="stat-mini card"><span class="stat-lbl">Metode Digunakan</span><span class="stat-val" style="color:var(--amber)">{reportData.length}</span></div>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">{#each Array(4) as _}<div class="skel" style="height:48px;"></div>{/each}</div>
  {:else if reportData.length === 0}
    <div class="empty-state">Tidak ada data untuk periode ini</div>
  {:else}
    <div class="content-grid">
      <div class="card" style="padding:20px;">
        <h3 style="margin-bottom:16px;">Distribusi Metode Bayar</h3>
        <div style="height:280px;"><canvas bind:this={chartCanvas}></canvas></div>
      </div>

      <div class="table-wrap" style="align-self:start;">
        <table class="table">
          <thead><tr><th>Metode</th><th>Transaksi</th><th>Total</th><th>%</th></tr></thead>
          <tbody>
            {#each reportData as row}
              {@const pct = totalRevenue > 0 ? ((row.total_amount / totalRevenue) * 100).toFixed(1) : '0'}
              <tr>
                <td>
                  <div style="display:flex;align-items:center;gap:8px;">
                    <CreditCard size={14} style="color:var(--brand-500);" />
                    <span style="font-weight:500;">{methodNames[row.method] ?? row.method}</span>
                  </div>
                </td>
                <td style="color:var(--text-2);">{row.transaction_count}</td>
                <td style="font-weight:600;">{currency(row.total_amount)}</td>
                <td>
                  <div style="display:flex;align-items:center;gap:8px;">
                    <div class="pct-bar"><div class="pct-fill" style="width:{pct}%"></div></div>
                    <span style="font-size:0.8125rem;color:var(--text-2);">{pct}%</span>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
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
  .content-grid { display:grid;grid-template-columns:1fr 1fr;gap:16px;align-items:start; }
  .pct-bar { flex:1;height:6px;background:var(--bg-muted);border-radius:var(--r-full);overflow:hidden;min-width:60px; }
  .pct-fill { height:100%;background:var(--brand-500);border-radius:var(--r-full); }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
  @media (max-width:700px) { .content-grid { grid-template-columns:1fr; } .stats-row { grid-template-columns:1fr; } }
</style>
