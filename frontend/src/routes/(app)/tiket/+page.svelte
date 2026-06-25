<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount, onDestroy } from 'svelte';
  import { relativeTime } from '$lib/utils/format';
  import { bluetooth } from '$lib/stores/bluetooth'; // bluetooth.connect / disconnect only
  import { ticketsApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { TicketScan } from '$lib/types';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { ScanLine, Camera, Keyboard, CheckCircle, XCircle } from 'lucide-svelte';

  let scanMode: 'camera' | 'manual' = 'manual';
  let manualCode = '';
  let scanning = false;
  let lastResult: { type: string; message: string } | null = null;
  let recentScans: TicketScan[] = [];
  let videoRef: HTMLVideoElement;
  let streamActive = false;
  let stream: MediaStream | null = null;

  onMount(() => {
    // Load recent scans from API if available
  });

  onDestroy(() => stopCamera());

  async function scanTicket(code: string) {
    if (!code.trim() || scanning) return;
    scanning = true;
    lastResult = null;
    try {
      const res = await ticketsApi.scan({ code: code.trim(), scan_method: scanMode });
      const r = res.data as any;
      lastResult = { type: r.result, message: r.message };
      const scan: TicketScan = {
        id: Date.now().toString(),
        ticket_id: code,
        officer_id: '',
        scan_method: scanMode,
        result: r.result,
        scanned_at: new Date().toISOString(),
      };
      recentScans = [scan, ...recentScans.slice(0, 19)];
      if (r.result === 'valid') toast.success('Tiket Valid', code);
      else toast.error('Tiket Tidak Valid', r.message);
    } catch (e: any) {
      toast.error('Gagal scan', e.message);
    } finally {
      scanning = false;
      manualCode = '';
    }
  }

  async function startCamera() {
    if (!navigator.mediaDevices) { toast.error('Kamera tidak tersedia'); return; }
    try {
      stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } });
      if (videoRef) { videoRef.srcObject = stream; streamActive = true; }
    } catch {
      toast.error('Akses kamera ditolak');
    }
  }

  function stopCamera() {
    stream?.getTracks().forEach(t => t.stop());
    if (videoRef) videoRef.srcObject = null;
    streamActive = false;
    stream = null;
  }

  function onModeChange(mode: 'camera' | 'manual') {
    scanMode = mode;
    if (mode === 'camera') startCamera();
    else stopCamera();
  }

  const resultConfig: Record<string, { label: string; variant: any; icon: any; ok: boolean }> = {
    valid:        { label:'VALID',          variant:'success', icon:CheckCircle, ok:true  },
    already_used: { label:'SUDAH DIPAKAI',  variant:'warning', icon:XCircle,    ok:false },
    expired:      { label:'KADALUARSA',     variant:'danger',  icon:XCircle,    ok:false },
    invalid:      { label:'TIDAK VALID',    variant:'danger',  icon:XCircle,    ok:false },
  };
</script>

<svelte:head><title>Scan Tiket — TiketKu</title></svelte:head>

<div class="scan-page">
  <div class="scan-panel card">
    <div class="scan-header">
      <div>
        <h2>Scan Tiket</h2>
        <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Validasi tiket pengunjung</p>
      </div>
      <div class="mode-switch">
        <button class="mode-btn" class:active={scanMode==='manual'} on:click={() => onModeChange('manual')} aria-pressed={scanMode==='manual'}>
          <Keyboard size={14} /> Manual
        </button>
        <button class="mode-btn" class:active={scanMode==='camera'} on:click={() => onModeChange('camera')} aria-pressed={scanMode==='camera'}>
          <Camera size={14} /> Kamera
        </button>
      </div>
    </div>

    {#if scanMode === 'camera'}
      <div class="camera-wrap">
        <video bind:this={videoRef} autoplay playsinline muted class="camera-feed" aria-label="Kamera scanner"></video>
        <div class="scan-overlay" aria-hidden="true">
          <div class="scan-frame"></div>
          <div class="scan-line"></div>
        </div>
        {#if !streamActive}
          <div class="camera-placeholder"><Camera size={40} style="color:var(--text-3);" /><p>Mengaktifkan kamera&hellip;</p></div>
        {/if}
      </div>
    {:else}
      <div class="manual-wrap">
        <div class="manual-icon" aria-hidden="true"><ScanLine size={48} style="color:var(--brand-500);" /></div>
        <form on:submit|preventDefault={() => scanTicket(manualCode)} style="width:100%;display:flex;gap:8px;">
          <input
            class="input input-lg"
            style="flex:1;font-family:var(--font-mono);letter-spacing:0.05em;"
            bind:value={manualCode}
            placeholder="Masukkan kode tiket atau scan barcode..."
            autocomplete="off"
            aria-label="Kode tiket"
            autofocus
          />
          <button class="btn btn-primary btn-lg" type="submit" disabled={scanning || !manualCode}>
            {#if scanning}<Spinner size={16} color="#fff" />{:else}<ScanLine size={16} />{/if}
            Scan
          </button>
        </form>
      </div>
    {/if}

    {#if lastResult}
      {@const cfg = resultConfig[lastResult.type] ?? resultConfig.invalid}
      <div class="scan-result" class:valid={cfg.ok} class:invalid={!cfg.ok}>
        <svelte:component this={cfg.icon} size={32} />
        <div>
          <p class="result-label">{cfg.label}</p>
          <p style="font-size:0.875rem;opacity:0.85;">{lastResult.message}</p>
        </div>
      </div>
    {/if}
  </div>

  <div class="recent-card card">
    <div style="padding:16px 20px;border-bottom:1px solid var(--border);">
      <h3>Riwayat Scan</h3>
      <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{recentScans.length} scan dalam sesi ini</p>
    </div>
    {#if recentScans.length === 0}
      <div style="padding:32px;text-align:center;color:var(--text-3);font-size:0.875rem;">Belum ada scan</div>
    {:else}
      <div class="table-wrap" style="border:none;border-radius:0;box-shadow:none;">
        <table class="table">
          <thead><tr><th>Kode</th><th>Metode</th><th>Hasil</th><th>Waktu</th></tr></thead>
          <tbody>
            {#each recentScans as s (s.id)}
              <tr>
                <td><span class="mono" style="font-size:0.8125rem;">{s.ticket_id}</span></td>
                <td style="text-transform:capitalize;color:var(--text-2);font-size:0.875rem;">{s.scan_method}</td>
                <td><Badge variant={resultConfig[s.result]?.variant ?? 'neutral'}>{resultConfig[s.result]?.label ?? s.result}</Badge></td>
                <td style="color:var(--text-2);font-size:0.8125rem;">{relativeTime(s.scanned_at)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>

<style>
  .scan-page { display:grid; grid-template-columns:1fr 1fr; gap:20px; max-width:1000px; align-items:start; }
  .scan-panel { overflow:hidden; }
  .scan-header { display:flex;align-items:flex-start;justify-content:space-between;padding:20px 20px 16px;border-bottom:1px solid var(--border); }
  .scan-header h2 { font-size:1.125rem; }
  .mode-switch { display:flex;background:var(--bg-muted);border-radius:var(--r-md);padding:3px;gap:2px; }
  .mode-btn { display:flex;align-items:center;gap:5px;padding:5px 10px;border-radius:var(--r-sm);font-size:0.8125rem;font-weight:500;color:var(--text-2);cursor:pointer;border:none;background:none;font-family:var(--font-sans);transition:all var(--ease-fast); }
  .mode-btn.active { background:var(--bg-surface);color:var(--text-1);box-shadow:var(--shadow-xs); }
  .camera-wrap { position:relative;height:280px;background:#000;overflow:hidden; }
  .camera-feed { width:100%;height:100%;object-fit:cover; }
  .scan-overlay { position:absolute;inset:0;display:flex;align-items:center;justify-content:center; }
  .scan-frame { width:180px;height:180px;border:2px solid rgba(255,255,255,0.7);border-radius:var(--r-lg);box-shadow:0 0 0 9999px rgba(0,0,0,0.45); }
  .scan-line { position:absolute;width:160px;height:2px;background:var(--brand-400);animation:scanLine 2s ease-in-out infinite; }
  @keyframes scanLine { 0%,100% { top:calc(50% - 80px); opacity:1; } 50% { top:calc(50% + 80px); opacity:0.6; } }
  .camera-placeholder { position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;color:var(--text-3);background:var(--bg-muted); }
  .manual-wrap { padding:24px 20px;display:flex;flex-direction:column;align-items:center;gap:20px; }
  .manual-icon { width:80px;height:80px;background:var(--brand-50);border-radius:var(--r-xl);display:flex;align-items:center;justify-content:center; }
  [data-theme='dark'] .manual-icon { background:rgba(99,102,241,0.12); }
  .scan-result { margin:16px;padding:16px;border-radius:var(--r-lg);display:flex;align-items:center;gap:14px;font-weight:600;font-size:1rem; }
  .scan-result.valid   { background:rgba(16,185,129,0.1);color:var(--emerald);border:1px solid rgba(16,185,129,0.25); }
  .scan-result.invalid { background:rgba(244,63,94,0.08);color:var(--rose);  border:1px solid rgba(244,63,94,0.2); }
  .result-label { font-size:1.125rem;font-weight:700; }
  .recent-card { overflow:hidden; }
  @media (max-width:768px) { .scan-page { grid-template-columns:1fr; } }
</style>
