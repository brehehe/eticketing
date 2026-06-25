import { writable, get } from 'svelte/store';
import type { BluetoothDevice, BluetoothState, PrintJob } from '$lib/types';
import { toast } from './toast';

// ── Web BT helper types ───────────────────────────────────────
type BtWebDevice = {
  id: string;
  name?: string;
  gatt?: any;
  addEventListener: Function;
};

function createBluetoothStore() {
  const devices    = writable<BluetoothDevice[]>([]);
  const activeDevice = writable<BluetoothDevice | null>(null);
  const state      = writable<BluetoothState>('disconnected');
  const printQueue = writable<PrintJob[]>([]);
  const isProcessing = writable(false);

  const STORAGE_KEY = 'tk-bt-device';

  // ── Which path is active ──────────────────────────────────
  // 'library' = @point-of-sale/webbluetooth-receipt-printer
  // 'manual'  = raw Web Bluetooth GATT
  let libPrinter: any = null;   // library instance
  let manualChar: any = null;   // GATT characteristic (manual path)
  let manualGatt: any = null;   // GATT server (manual path)
  let activePath: 'library' | 'manual' | null = null;

  // ── Persist last device ───────────────────────────────────
  function saveDevice(dev: { id: string; name: string }) {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(dev)); } catch { /* ignore */ }
  }
  function loadDevice(): { id: string; name: string } | null {
    try { return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null'); } catch { return null; }
  }

  // ── Known BLE service UUIDs for thermal printers ─────────
  const PRINTER_SERVICES = [
    '000018f0-0000-1000-8000-00805f9b34fb', // Generic custom (Star, etc.)
    '0000e0ff-0000-1000-8000-00805f9b34fb', // Custom raw
    '0000ff00-0000-1000-8000-00805f9b34fb', // Rongta FF00
    '0000ffe0-0000-1000-8000-00805f9b34fb', // HM-10
    '49535343-fe7d-4ae5-8fa9-9fafd205e455', // ISSC BLE
    'e7810a71-73ae-499d-8c15-faa9aef0c3f2', // Xprinter / Epson BLE
    '0000ae30-0000-1000-8000-00805f9b34fb', // Custom thermal
    '00001101-0000-1000-8000-00805f9b34fb', // SPP (may appear in BLE fallback)
  ];

  // ── Helper: set app device state ─────────────────────────
  function setConnected(id: string, name: string) {
    const dev: BluetoothDevice = {
      id, name, address: id, type: 'printer', state: 'connected',
      last_connected: new Date().toISOString(), is_default: true,
    };
    activeDevice.set(dev);
    state.set('connected');
    saveDevice({ id, name });
    toast.success('Printer terhubung', name);
    processQueue();
  }

  function handleDisconnect() {
    state.set('disconnected');
    activeDevice.set(null);
    manualChar = null;
    manualGatt = null;
    activePath = null;
    toast.warning('Printer terputus');
  }

  // ── Path 1: @point-of-sale/webbluetooth-receipt-printer ──
  async function connectViaLibrary(): Promise<boolean> {
    try {
      const mod = await import('@point-of-sale/webbluetooth-receipt-printer');
      const Ctor = (mod as any).WebBluetoothReceiptPrinter ?? mod.default;
      if (!Ctor) return false;

      if (!libPrinter) {
        libPrinter = new Ctor();

        libPrinter.addEventListener('connected', (info: any) => {
          const name = info?.name ?? info?.device?.name ?? 'Printer';
          const id   = info?.id   ?? info?.device?.id   ?? 'lib';
          activePath = 'library';
          setConnected(id, name);
        });

        libPrinter.addEventListener('disconnected', () => {
          if (activePath === 'library') handleDisconnect();
        });
      }

      // library.connect() shows the browser Bluetooth picker itself
      await libPrinter.connect();
      return true;
    } catch (e: any) {
      // Re-throw cancellations so outer catch can handle them
      if (['NotFoundError','AbortError','NotAllowedError','SecurityError']
          .includes(e?.name) || String(e).includes('cancel')) {
        throw e;
      }
      console.warn('[BT] library path failed, trying manual:', e);
      return false;
    }
  }

  // ── Path 2: raw Web Bluetooth GATT ───────────────────────
  async function connectViaManual(hint?: BluetoothDevice) {
    const btDevice: BtWebDevice = hint?.name
      ? await (navigator as any).bluetooth.requestDevice({
          filters: [{ name: hint.name }],
          optionalServices: PRINTER_SERVICES,
        })
      : await (navigator as any).bluetooth.requestDevice({
          acceptAllDevices: true,
          optionalServices: PRINTER_SERVICES,
        });

    btDevice.addEventListener('gattserverdisconnected', handleDisconnect);

    manualGatt = await btDevice.gatt.connect();

    // Try to discover a writable characteristic from any service
    let foundChar: any = null;
    try {
      const services = await manualGatt.getPrimaryServices();
      for (const svc of services) {
        try {
          const chars = await svc.getCharacteristics();
          for (const c of chars) {
            if (c.properties.write || c.properties.writeWithoutResponse) {
              foundChar = c;
              break;
            }
          }
        } catch { /* skip unreachable service */ }
        if (foundChar) break;
      }
    } catch { /* getPrimaryServices() may throw on some browsers */ }

    // If auto-discovery failed, try known UUIDs explicitly
    if (!foundChar) {
      const FALLBACKS: [string, string][] = [
        ['000018f0-0000-1000-8000-00805f9b34fb', '00002af1-0000-1000-8000-00805f9b34fb'],
        ['0000ff00-0000-1000-8000-00805f9b34fb', '0000ff02-0000-1000-8000-00805f9b34fb'],
        ['0000ffe0-0000-1000-8000-00805f9b34fb', '0000ffe1-0000-1000-8000-00805f9b34fb'],
        ['e7810a71-73ae-499d-8c15-faa9aef0c3f2', 'bef8d6c9-9c21-4c9e-b632-bd58c1009f9f'],
      ];
      for (const [svcUuid, charUuid] of FALLBACKS) {
        try {
          const svc = await manualGatt.getPrimaryService(svcUuid);
          foundChar = await svc.getCharacteristic(charUuid);
          break;
        } catch { /* try next */ }
      }
    }

    if (!foundChar) {
      manualGatt.disconnect();
      throw new Error(
        'Tidak ditemukan karakteristik tulis di printer ini. ' +
        'Printer mungkin menggunakan Bluetooth Classic (SPP), bukan BLE.'
      );
    }

    manualChar = foundChar;
    activePath = 'manual';
    setConnected(btDevice.id, btDevice.name ?? 'Printer');
  }

  // ── Main connect entry point ──────────────────────────────
  async function connect(device?: BluetoothDevice) {
    if (!('bluetooth' in navigator)) {
      const msg = window.isSecureContext
        ? 'Browser tidak mendukung Web Bluetooth API. Gunakan Chrome atau Edge.'
        : 'Web Bluetooth memerlukan HTTPS.';
      toast.error('Bluetooth tidak tersedia', msg);
      return;
    }

    state.set('connecting');
    try {
      // Try library path first (handles connected/disconnected events internally)
      const libOk = await connectViaLibrary();
      if (!libOk) {
        // Library not available → use raw GATT
        await connectViaManual(device);
      }
    } catch (err: any) {
      const isCancelled = ['NotFoundError','AbortError','NotAllowedError','SecurityError']
        .some(n => err?.name === n) || String(err).toLowerCase().includes('cancel');

      if (isCancelled) {
        state.set('disconnected');
        toast.info('Dibatalkan', 'Pencarian printer dibatalkan.');
      } else {
        state.set('error');
        toast.error('Gagal terhubung', String(err));
      }
    }
  }

  // ── Auto-reconnect via library ────────────────────────────
  async function reconnect(saved: { id: string; name: string }) {
    try {
      const mod = await import('@point-of-sale/webbluetooth-receipt-printer');
      const Ctor = (mod as any).WebBluetoothReceiptPrinter ?? mod.default;
      if (!Ctor) return;
      if (!libPrinter) libPrinter = new Ctor();
      await libPrinter.reconnect(saved);
    } catch { /* silent */ }
  }

  // ── Send raw bytes to printer ─────────────────────────────
  async function printRaw(data: Uint8Array): Promise<void> {
    if (activePath === 'library' && libPrinter) {
      await libPrinter.print(data);
      return;
    }
    if (activePath === 'manual' && manualChar) {
      const CHUNK = 512;
      for (let i = 0; i < data.length; i += CHUNK) {
        try {
          await manualChar.writeValueWithoutResponse(data.slice(i, i + CHUNK));
        } catch {
          await manualChar.writeValue(data.slice(i, i + CHUNK));
        }
        await new Promise(r => setTimeout(r, 20));
      }
      return;
    }
    throw new Error('Printer tidak terhubung');
  }

  // ── Build ESC/POS bytes ───────────────────────────────────
  async function buildEncodedData(lines: string[]): Promise<Uint8Array> {
    try {
      const { ReceiptPrinterEncoder } = await import('@point-of-sale/receipt-printer-encoder');
      const encoder = new (ReceiptPrinterEncoder as any)({ language: 'esc-pos' });
      let e = encoder.initialize();
      for (const line of lines) { e = e.text(line).newline(); }
      return e.cut().encode();
    } catch {
      return buildRawEscPos(lines);
    }
  }

  function buildRawEscPos(lines: string[]): Uint8Array {
    const ESC = 0x1b, GS = 0x1d, LF = 0x0a;
    const enc = new TextEncoder();
    const bytes: number[] = [ESC, 0x40];
    for (const line of lines) { bytes.push(...Array.from(enc.encode(line)), LF); }
    bytes.push(LF, LF, LF, ESC, 0x69, GS, 0x56, 0x00, GS, 0x56, 0x41, 0x00);
    return new Uint8Array(bytes);
  }

  // ── Print queue ───────────────────────────────────────────
  async function processQueue() {
    if (get(isProcessing)) return;
    isProcessing.set(true);
    const queue = get(printQueue);
    for (const job of queue) {
      printQueue.update(q => q.map(j => j.id === job.id ? { ...j, status: 'printing' } : j));
      try {
        let data: Uint8Array;
        if ((job.payload as any)?.rawBytes instanceof Uint8Array) {
          data = (job.payload as any).rawBytes;
        } else {
          data = await buildEncodedData(buildJobLines(job));
        }
        await printRaw(data);
        printQueue.update(q => q.map(j => j.id === job.id ? { ...j, status: 'done' } : j));
        setTimeout(() => printQueue.update(q => q.filter(j => j.id !== job.id)), 2000);
      } catch {
        const retries = job.retries + 1;
        if (retries >= 3) {
          printQueue.update(q => q.map(j => j.id === job.id ? { ...j, status: 'failed', retries } : j));
          toast.error('Print gagal', 'Job gagal setelah 3 percobaan');
        } else {
          printQueue.update(q => q.map(j => j.id === job.id ? { ...j, status: 'queued', retries } : j));
        }
      }
    }
    isProcessing.set(false);
  }

  function buildJobLines(job: PrintJob): string[] {
    if (job.type === 'receipt') {
      const p = job.payload as import('$lib/types').ReceiptPayload;
      return [
        p.venue.name,
        p.venue.address ?? '',
        '--------------------------------',
        `No. Nota: ${p.transaction.invoice}`,
        `Tanggal: ${new Date(p.transaction.created_at).toLocaleString('id-ID')}`,
        '--------------------------------',
        ...(p.transaction.items ?? []).map(i =>
          `${i.product?.name ?? 'Item'} x${i.qty}  Rp${i.subtotal.toLocaleString('id-ID')}`
        ),
        '--------------------------------',
        `TOTAL  Rp${p.transaction.total.toLocaleString('id-ID')}`,
        '',
        'Terima kasih!',
      ];
    } else {
      const p = job.payload as import('$lib/types').TicketPayload;
      return [
        p.venue.name,
        '================================',
        p.ticket.product?.name ?? 'Tiket',
        `No: ${p.ticket.code}`,
        p.ticket.valid_until
          ? `Berlaku: ${new Date(p.ticket.valid_until).toLocaleDateString('id-ID')}`
          : 'Sekali Pakai',
        '================================',
        'Tunjukkan ke petugas',
      ];
    }
  }

  function enqueue(job: Omit<PrintJob, 'id' | 'status' | 'retries'>): string {
    const newJob: PrintJob = {
      ...job, id: Math.random().toString(36).slice(2),
      status: 'queued', retries: 0, created_at: new Date().toISOString(),
    };
    printQueue.update(q => [...q, newJob]);
    if (get(state) === 'connected') processQueue();
    return newJob.id;
  }

  function disconnect() {
    try { libPrinter?.disconnect?.(); } catch { /* ignore */ }
    try { manualGatt?.disconnect(); } catch { /* ignore */ }
    state.set('disconnected');
    activeDevice.set(null);
    manualChar = null; manualGatt = null; activePath = null;
    try { localStorage.removeItem(STORAGE_KEY); } catch { /* ignore */ }
  }

  function forceReset() {
    state.set('disconnected');
    manualChar = null; manualGatt = null; activePath = null;
  }

  // Auto-reconnect on page load
  if (typeof window !== 'undefined') {
    window.addEventListener('load', async () => {
      const saved = loadDevice();
      if (saved) await reconnect(saved);
    });
  }

  return {
    devices:     { subscribe: devices.subscribe },
    activeDevice:{ subscribe: activeDevice.subscribe },
    state:       { subscribe: state.subscribe },
    printQueue:  { subscribe: printQueue.subscribe },
    isProcessing:{ subscribe: isProcessing.subscribe },
    connect,
    reconnect,
    disconnect,
    forceReset,
    enqueue,
    printRaw,
    retryJob: (id: string) => {
      printQueue.update(q => q.map(j => j.id === id ? { ...j, status: 'queued', retries: 0 } : j));
      processQueue();
    },
    cancelJob: (id: string) => printQueue.update(q => q.filter(j => j.id !== id)),
  };
}

export const bluetooth = createBluetoothStore();

export const btState      = { subscribe: bluetooth.state.subscribe };
export const btDevice     = { subscribe: bluetooth.activeDevice.subscribe };
export const btDevices    = { subscribe: bluetooth.devices.subscribe };
export const btPrintQueue = { subscribe: bluetooth.printQueue.subscribe };
export const btProcessing = { subscribe: bluetooth.isProcessing.subscribe };
