<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;

  import { bluetooth, btState, btDevice, btPrintQueue, btProcessing } from '$lib/stores/bluetooth';
  import { toast } from '$lib/stores/toast';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import {
    Bluetooth, BluetoothOff, X, RefreshCw
  } from 'lucide-svelte';

  let testing = false;

  async function doBleTestPrint() {
    testing = true;
    try {
      const ESC = 0x1b, GS = 0x1d, LF = 0x0a;
      const enc = new TextEncoder();
      const bytes: number[] = [ESC, 0x40]; // Initialize
      bytes.push(...Array.from(enc.encode("TEST PRINT BLUETOOTH BLE\n")), LF);
      bytes.push(...Array.from(enc.encode("Printer Thermal Berhasil Terhubung!\n")), LF);
      bytes.push(LF, LF, LF, ESC, 0x69, GS, 0x56, 0x00, GS, 0x56, 0x41, 0x00); // Cut/feed
      const rawBytes = new Uint8Array(bytes);

      bluetooth.enqueue({
        type: 'receipt',
        payload: {
          rawBytes,
          venue: { name: 'Test Print BLE' },
          transaction: { invoice: 'TEST-BLE', created_at: new Date().toISOString(), total: 0 }
        } as any
      });
      toast.success('Test print dikirim', 'Periksa apakah printer mencetak');
    } catch (e: any) {
      toast.error('Test print gagal', e.message);
    } finally {
      testing = false;
    }
  }

  $: btSt    = $btState;
  $: btDev   = $btDevice;
  $: queue   = $btPrintQueue;

  const btStateConf = {
    connected:    { label: 'Terhubung',       variant: 'success' as const },
    connecting:   { label: 'Menghubungkan…',  variant: 'warning' as const },
    disconnected: { label: 'Tidak Terhubung', variant: 'neutral' as const },
    error:        { label: 'Error',           variant: 'danger'  as const },
  };

  const jobConf: Record<string, { variant: any; label: string }> = {
    queued:   { variant: 'neutral', label: 'Menunggu' },
    printing: { variant: 'warning', label: 'Mencetak' },
    done:     { variant: 'success', label: 'Selesai'  },
    failed:   { variant: 'danger',  label: 'Gagal'    },
  };

  $: btCfg  = btStateConf[btSt]  ?? btStateConf.disconnected;

  const supportsBt      = typeof window !== 'undefined' && 'bluetooth' in navigator;
</script>

<svelte:head><title>Printer — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Manajemen Printer</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Cetak nota & tiket langsung ke printer thermal via Bluetooth BLE</p>
    </div>
  </div>

  <div class="card" style="padding:24px;">
    <div style="display:flex;align-items:center;gap:16px;">
      <div class="printer-icon" class:connected={btSt === 'connected'}>
        {#if btSt === 'connected'}<Bluetooth size={24} />{:else}<BluetoothOff size={24} />{/if}
      </div>
      <div style="flex:1;">
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
          <span style="font-weight:600;font-size:0.9375rem;">{btDev?.name ?? 'Printer BLE'}</span>
          <Badge variant={btCfg.variant} dot pulse={btSt === 'connecting'}>{btCfg.label}</Badge>
        </div>
        <p style="font-size:0.8125rem;color:var(--text-3);">
          {btDev ? btDev.address : 'Belum ada perangkat BLE terhubung'}
        </p>
      </div>
    </div>

    <div style="display:flex;gap:8px;margin-top:20px;">
      {#if btSt === 'connected'}
        <button class="btn btn-primary" style="gap:6px;" on:click={doBleTestPrint} disabled={testing}>
          {#if testing}<Spinner size={14} color="#fff" />{:else}<Bluetooth size={14} />{/if} Test Print
        </button>
        <button class="btn btn-secondary" style="gap:6px;" on:click={() => bluetooth.disconnect()}>
          <BluetoothOff size={14} /> Putuskan
        </button>
      {:else if btSt === 'connecting'}
        <button class="btn btn-primary" style="gap:6px;" disabled>
          <Spinner size={14} color="#fff" /> Menghubungkan...
        </button>
        <button class="btn btn-secondary" style="gap:6px;" on:click={() => bluetooth.forceReset()}>
          <X size={14} /> Batal
        </button>
      {:else}
        <button class="btn btn-primary" style="gap:6px;"
          disabled={!supportsBt}
          on:click={() => bluetooth.connect()}
        >
          <Bluetooth size={14} /> Cari Printer BLE
        </button>
        {#if btSt === 'error'}
          <button class="btn btn-secondary" style="gap:6px;" on:click={() => bluetooth.forceReset()}>
            <RefreshCw size={14} /> Reset
          </button>
        {/if}
      {/if}
    </div>
  </div>

  {#if !supportsBt}
    <div class="hint-box warning">
      Browser tidak mendukung Web Bluetooth API atau koneksi tidak aman (non-HTTPS). Gunakan Chrome, Edge, atau browser yang didukung via HTTPS.
    </div>
  {/if}

  <!-- BLE print queue -->
  {#if queue.length > 0}
    <div class="card">
      <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;justify-content:space-between;">
        <span style="font-weight:600;font-size:0.875rem;">Antrian Cetak BLE ({queue.length})</span>
        {#if $btProcessing}
          <div style="display:flex;align-items:center;gap:6px;font-size:0.8125rem;color:var(--text-2);">
            <Spinner size={13} /> Memproses...
          </div>
        {/if}
      </div>
      <div class="table-wrap" style="border:none;border-radius:0;box-shadow:none;">
        <table class="table">
          <thead>
            <tr><th>Job</th><th>Tipe</th><th>Status</th><th>Retry</th><th></th></tr>
          </thead>
          <tbody>
            {#each queue as job (job.id)}
              <tr>
                <td><span class="mono" style="font-size:0.8rem;">{job.id.slice(0,8)}…</span></td>
                <td style="text-transform:capitalize;">{job.type}</td>
                <td><Badge variant={jobConf[job.status]?.variant ?? 'neutral'}>{jobConf[job.status]?.label ?? job.status}</Badge></td>
                <td style="color:var(--text-2);">{job.retries}</td>
                <td>
                  <div style="display:flex;gap:4px;">
                    {#if job.status === 'failed'}
                      <button class="btn btn-ghost btn-icon btn-sm" on:click={() => bluetooth.retryJob(job.id)}>
                        <RefreshCw size={13} />
                      </button>
                    {/if}
                    <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);"
                      on:click={() => bluetooth.cancelJob(job.id)}>
                      <X size={13} />
                    </button>
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
  .page { display:flex;flex-direction:column;gap:16px;max-width:1400px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }

  /* Printer icon */
  .printer-icon {
    width:52px;height:52px;flex-shrink:0;
    border-radius:var(--r-xl);
    background:var(--bg-muted);
    display:flex;align-items:center;justify-content:center;
    color:var(--text-3);
    transition:all var(--ease-base);
  }
  .printer-icon.connected { background:rgba(16,185,129,0.1);color:var(--emerald); }

  /* Hint boxes */
  .hint-box {
    padding:10px 14px;border-radius:var(--r-md);
    font-size:0.8125rem;line-height:1.5;
  }
  .hint-box.warning { background:rgba(245,158,11,0.07);border:1px solid rgba(245,158,11,0.2);color:var(--amber); }
</style>
