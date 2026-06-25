<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;

  import { onMount, onDestroy } from 'svelte';
  import { bluetooth, btState, btDevice, btPrintQueue, btProcessing } from '$lib/stores/bluetooth';
  import { serial, serialState, serialPortName, serialBaud, serialError } from '$lib/stores/serial';
  import { bridge, bridgeState, bridgePortName } from '$lib/stores/bridge';
  import { toast } from '$lib/stores/toast';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import {
    Bluetooth, BluetoothOff, Printer, RefreshCw, Usb, Server,
    X, CheckCircle, XCircle, Clock, Zap, AlertTriangle, Terminal
  } from 'lucide-svelte';

  let testing      = false;
  let bridgeTesting = false;

  async function doTestPrint() {
    testing = true;
    try {
      await serial.testPrint();
      toast.success('Test print dikirim', 'Periksa apakah printer mencetak');
    } catch (e: any) {
      toast.error('Test print gagal', e.message);
    } finally {
      testing = false;
    }
  }

  async function doBridgeTest() {
    bridgeTesting = true;
    try {
      await bridge.testPrint();
      toast.success('Test print via Bridge berhasil!', 'Printer mencetak via printbridge.js');
    } catch (e: any) {
      toast.error('Bridge test gagal', e.message);
    } finally {
      bridgeTesting = false;
    }
  }

  $: btSt    = $btState;
  $: btDev   = $btDevice;
  $: queue   = $btPrintQueue;
  $: srlSt   = $serialState;
  $: srlPort = $serialPortName;
  $: srlBaud = $serialBaud;
  $: srlErr  = $serialError;
  $: brgSt   = $bridgeState;
  $: brgPort = $bridgePortName;

  let selectedBaud = 115200;
  const baudOptions = [
    { value: 115200, label: '115200 (default)' },
    { value: 9600,   label: '9600' },
    { value: 38400,  label: '38400' },
    { value: 57600,  label: '57600' },
    { value: 19200,  label: '19200' },
  ];

  // Which tab is active
  let activeTab: 'bridge' | 'serial' | 'ble' = 'bridge';

  const btStateConf = {
    connected:    { label: 'Terhubung',       variant: 'success' as const },
    connecting:   { label: 'Menghubungkan…',  variant: 'warning' as const },
    disconnected: { label: 'Tidak Terhubung', variant: 'neutral' as const },
    error:        { label: 'Error',           variant: 'danger'  as const },
  };
  const srlStateConf = {
    connected:    { label: 'Terhubung',       variant: 'success' as const },
    connecting:   { label: 'Membuka Port…',   variant: 'warning' as const },
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
  $: srlCfg = srlStateConf[srlSt] ?? srlStateConf.disconnected;

  const supportsSerial  = typeof window !== 'undefined' && 'serial'    in navigator;
  const supportsBt      = typeof window !== 'undefined' && 'bluetooth' in navigator;

  let checkInterval: any;
  onMount(() => {
    bridge.checkStatus();
    checkInterval = setInterval(() => {
      bridge.checkStatus();
    }, 3000);
  });

  onDestroy(() => {
    if (checkInterval) clearInterval(checkInterval);
  });
</script>

<svelte:head><title>Printer — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Manajemen Printer</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Cetak nota & tiket langsung ke printer thermal</p>
    </div>
  </div>

  <!-- Tab switcher -->
  <div class="tabs">
    <button
      class="tab" class:active={activeTab === 'bridge'}
      on:click={() => (activeTab = 'bridge')}
    >
      <Server size={15} />
      <span>Print Bridge</span>
      {#if brgSt === 'online'}<div class="dot dot-success"></div>{/if}
    </button>
    <button
      class="tab" class:active={activeTab === 'serial'}
      on:click={() => (activeTab = 'serial')}
    >
      <Usb size={15} />
      <span>Serial / Bluetooth Classic</span>
      {#if srlSt === 'connected'}<div class="dot dot-success"></div>{/if}
    </button>
    <button
      class="tab" class:active={activeTab === 'ble'}
      on:click={() => (activeTab = 'ble')}
    >
      <Bluetooth size={15} />
      <span>Bluetooth BLE</span>
      {#if btSt === 'connected'}<div class="dot dot-success"></div>{/if}
    </button>
  </div>

  <!-- ── BRIDGE TAB ──────────────────────────────────────── -->
  {#if activeTab === 'bridge'}
    <div class="card" style="padding:24px;">
      <!-- Recommended badge -->
      <div class="rec-badge">
        <Zap size={13} /> Sangat Direkomendasikan (Stabil & Cepat)
      </div>

      <div style="display:flex;align-items:center;gap:16px;margin-top:16px;">
        <div class="printer-icon" class:connected={brgSt === 'online'}>
          <Server size={24} />
        </div>
        <div style="flex:1;">
          <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
            <span style="font-weight:600;font-size:0.9375rem;">
              Local Print Bridge (printbridge.js)
            </span>
            <Badge variant={brgSt === 'online' ? 'success' : 'danger'} dot>
              {brgSt === 'online' ? 'Online' : 'Offline'}
            </Badge>
          </div>
          <p style="font-size:0.8125rem;color:var(--text-3);line-height:1.5;">
            {#if brgSt === 'online'}
              Menghubungkan ke port printer: <strong style="color:var(--text-1);">{brgPort || '/dev/cu.RPP02N'}</strong>
            {:else}
              Server printbridge.js mati/offline di port 9100.
            {/if}
          </p>
        </div>
      </div>

      <div style="display:flex;gap:8px;margin-top:20px;flex-wrap:wrap;align-items:center;">
        {#if brgSt === 'online'}
          <button class="btn btn-primary" style="gap:6px;" on:click={doBridgeTest} disabled={bridgeTesting}>
            {#if bridgeTesting}<Spinner size={14} color="#fff" />{:else}<Printer size={14} />{/if} Test Print via Bridge
          </button>
        {/if}
        <button class="btn btn-secondary" style="gap:6px;" on:click={() => bridge.checkStatus()}>
          <RefreshCw size={14} /> Periksa Status
        </button>
      </div>
    </div>

    <!-- Setup guide for Bridge -->
    <div class="guide-card" style="margin-top:16px; margin-bottom:16px;">
      <h3 style="font-size:0.9375rem;font-weight:600;margin-bottom:14px;">
        ⚙️ Cara Menjalankan Local Print Bridge (macOS / Linux / Windows)
      </h3>
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <div>Pastikan printer thermal RPP02N menyala dan sudah terhubung via Bluetooth (paired) di System Settings.</div>
        </div>
        <div class="step">
          <div class="step-num">2</div>
          <div>Buka terminal baru di folder project e-ticketing dan jalankan:
            <pre style="background:var(--bg-muted);padding:8px 12px;border-radius:6px;font-family:monospace;font-size:0.8rem;margin:8px 0;border:1px solid var(--border);overflow-x:auto;color:var(--text-1);">node printbridge.js</pre>
          </div>
        </div>
        <div class="step">
          <div class="step-num">3</div>
          <div>
            Jika nama serial port Bluetooth printer Anda berbeda, jalankan dengan environment variable <code>PRINTER_PORT</code>:
            <pre style="background:var(--bg-muted);padding:8px 12px;border-radius:6px;font-family:monospace;font-size:0.8rem;margin:8px 0;border:1px solid var(--border);overflow-x:auto;color:var(--text-1);">PRINTER_PORT=/dev/cu.RPP02N-SPP node printbridge.js</pre>
          </div>
        </div>
        <div class="step">
          <div class="step-num">4</div>
          <div>Setelah status di atas berubah menjadi <strong style="color:var(--emerald);">Online</strong>, printer siap mencetak struk secara instan dari halaman Kasir!</div>
        </div>
      </div>
    </div>

  <!-- ── SERIAL TAB ──────────────────────────────────────── -->
  {:else if activeTab === 'serial'}
    <div class="card" style="padding:24px;">
      <!-- Recommended badge -->
      <div class="rec-badge">
        <Zap size={13} /> Direkomendasikan untuk RPP02N
      </div>

      <div style="display:flex;align-items:center;gap:16px;margin-top:16px;">
        <div class="printer-icon" class:connected={srlSt === 'connected'}>
          <Usb size={24} />
        </div>
        <div style="flex:1;">
          <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
            <span style="font-weight:600;font-size:0.9375rem;">
              {srlSt === 'connected' ? srlPort || 'Printer Terhubung' : 'Web Serial Port'}
            </span>
            <Badge variant={srlCfg.variant} dot pulse={srlSt === 'connecting'}>{srlCfg.label}</Badge>
          </div>
          <p style="font-size:0.8125rem;color:var(--text-3);line-height:1.5;">
            {srlSt === 'connected'
              ? 'Printer siap menerima data cetak'
              : 'Cocok untuk RPP02N via Bluetooth Classic atau USB'}
          </p>
        </div>
      </div>

      <div style="display:flex;gap:8px;margin-top:20px;flex-wrap:wrap;align-items:center;">
        {#if srlSt === 'connected'}
          <button class="btn btn-primary" style="gap:6px;" on:click={doTestPrint} disabled={testing}>
            {#if testing}<Spinner size={14} color="#fff" />{:else}<Printer size={14} />{/if} Test Print
          </button>
          <button class="btn btn-secondary" style="gap:6px;" on:click={() => serial.disconnect()}>
            <X size={14} /> Putuskan
          </button>
          <span style="font-size:0.8125rem;color:var(--text-3);">@ {srlBaud} baud</span>
        {:else if srlSt === 'connecting'}
          <button class="btn btn-primary" style="gap:6px;" disabled>
            <Spinner size={14} color="#fff" /> Membuka port...
          </button>
          <button class="btn btn-secondary" style="gap:6px;" on:click={() => serial.forceReset()}>
            <X size={14} /> Batal
          </button>
        {:else}
          <div class="field" style="margin:0;">
            <Select
              id="baud-select"
              bind:value={selectedBaud}
              options={baudOptions}
            />
          </div>
          <button
            class="btn btn-primary"
            style="gap:6px;"
            disabled={!supportsSerial}
            on:click={() => serial.connect(selectedBaud)}
          >
            <Usb size={14} /> Pilih Port Serial
          </button>
          {#if srlSt === 'error'}
            <button class="btn btn-secondary" style="gap:6px;" on:click={() => serial.forceReset()}>
              <RefreshCw size={14} /> Reset
            </button>
          {/if}
        {/if}
      </div>

      {#if srlSt === 'error' && srlErr}
        <div class="hint-box warning" style="margin-top:14px;">
          <div style="display:flex;gap:6px;align-items:flex-start;">
            <AlertTriangle size={14} style="flex-shrink:0;margin-top:1px;" />
            <div>
              <strong>Troubleshooting:</strong>
              <ol style="margin:6px 0 0 16px;padding:0;line-height:1.8;">
                <li>Pastikan printer <strong>menyala</strong> dan tidak dalam mode sleep</li>
                <li>Coba <strong>restart printer</strong> → tunggu 10 detik → klik Pilih Port lagi</li>
                <li>Coba baud rate berbeda (9600 sering dipakai RPP02N)</li>
                <li>Jika port tidak muncul: buka <strong>System Settings → Bluetooth</strong> → Remove RPP02N → Pair ulang</li>
              </ol>
            </div>
          </div>
        </div>
      {/if}

      {#if !supportsSerial}
        <div class="hint-box warning" style="margin-top:14px;">
          Web Serial API tidak tersedia di browser ini. Gunakan Chrome atau Edge versi 89+.
        </div>
      {/if}
    </div>

    <!-- Setup guide -->
    <div class="guide-card">
      <h3 style="font-size:0.9375rem;font-weight:600;margin-bottom:14px;">
        📋 Cara Setup RPP02N (Bluetooth Classic → Serial Port)
      </h3>
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <div>Buka <strong>System Settings → Bluetooth</strong> di Mac → pastikan RPP02N sudah di-pair (muncul di daftar "My Devices")</div>
        </div>
        <div class="step">
          <div class="step-num">2</div>
          <div>Setelah paired, macOS otomatis membuat virtual serial port: <code>/dev/cu.RPP02N-SPP</code> atau serupa</div>
        </div>
        <div class="step">
          <div class="step-num">3</div>
          <div>Kembali ke halaman ini → klik <strong>"Pilih Port Serial"</strong> → pilih port RPP02N dari daftar</div>
        </div>
        <div class="step">
          <div class="step-num">4</div>
          <div>Setelah terhubung, cetak nota/tiket dari halaman <strong>Kasir</strong> — data langsung terkirim ke printer</div>
        </div>
      </div>
    </div>

  <!-- ── BLE TAB ──────────────────────────────────────────── -->
  {:else}
    <div class="card" style="padding:24px;">
      <div class="hint-box info" style="margin-bottom:18px;">
        Tab ini untuk printer BLE (bukan Classic Bluetooth). RPP02N menggunakan Classic BT — gunakan tab Serial di atas.
      </div>

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
  {/if}
</div>

<style>
  .page { display:flex;flex-direction:column;gap:16px;max-width:720px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }

  /* Tabs */
  .tabs {
    display:flex;gap:4px;
    background:var(--bg-subtle);
    border:1px solid var(--border);
    border-radius:var(--r-lg);
    padding:4px;
  }
  .tab {
    flex:1;
    display:flex;align-items:center;justify-content:center;gap:7px;
    padding:9px 14px;
    border-radius:var(--r-md);
    font-size:0.875rem;font-weight:500;
    color:var(--text-2);
    transition:all var(--ease-fast);
    position:relative;
  }
  .tab:hover { color:var(--text-1); }
  .tab.active {
    background:var(--bg-elevated);
    color:var(--text-1);
    box-shadow:var(--shadow-sm);
    font-weight:600;
  }
  .dot {
    width:7px;height:7px;border-radius:50%;
  }
  .dot-success { background:var(--emerald); }

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

  /* Recommended badge */
  .rec-badge {
    display:inline-flex;align-items:center;gap:5px;
    background:rgba(16,185,129,0.1);
    color:var(--emerald);
    border:1px solid rgba(16,185,129,0.2);
    font-size:0.75rem;font-weight:600;
    padding:3px 10px;border-radius:var(--r-full);
  }

  /* Hint boxes */
  .hint-box {
    padding:10px 14px;border-radius:var(--r-md);
    font-size:0.8125rem;line-height:1.5;
  }
  .hint-box.warning { background:rgba(245,158,11,0.07);border:1px solid rgba(245,158,11,0.2);color:var(--amber); }
  .hint-box.info    { background:rgba(99,102,241,0.06);border:1px solid rgba(99,102,241,0.2);color:var(--indigo,var(--brand-500)); }

  /* Guide */
  .guide-card {
    background:var(--bg-subtle);border:1px solid var(--border);
    border-radius:var(--r-lg);padding:20px 24px;
  }
  .steps { display:flex;flex-direction:column;gap:12px; }
  .step { display:flex;align-items:flex-start;gap:12px;font-size:0.875rem;line-height:1.6; }
  .step-num {
    flex-shrink:0;width:24px;height:24px;border-radius:50%;
    background:var(--brand-500);color:#fff;
    font-size:0.75rem;font-weight:700;
    display:flex;align-items:center;justify-content:center;margin-top:1px;
  }
  code {
    font-family:var(--font-mono);font-size:0.8125rem;
    background:var(--bg-muted);padding:1px 5px;border-radius:4px;
  }
</style>
