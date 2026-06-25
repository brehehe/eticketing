import { writable, get } from 'svelte/store';
import { toast } from './toast';

export type SerialState = 'disconnected' | 'connecting' | 'connected' | 'error';

function createSerialStore() {
  const state    = writable<SerialState>('disconnected');
  const portName = writable<string>('');
  const baudRate = writable<number>(115200);
  const errorMsg = writable<string>('');

  let port: any     = null;
  let reader: any   = null; // keeps readable stream open → establishes RFCOMM channel
  let connecting    = false; // guard against concurrent connect() calls

  // ── Open port with specific baud rate ────────────────────
  async function openFresh(p: any, baud: number): Promise<void> {
    // Force-close if already open from a previous session
    if (p.readable !== null) {
      console.log('[Serial] Closing stale open port before reopen...');
      try {
        if (reader) { await reader.cancel(); reader = null; }
        await p.close();
      } catch { /* ignore */ }
      await new Promise(r => setTimeout(r, 300));
    }
    await p.open({
      baudRate:    baud,
      dataBits:    8,
      stopBits:    1,
      parity:      'none',
      bufferSize:  4096,
      flowControl: 'none',
    });
  }

  // ── Main connect ──────────────────────────────────────────
  async function connect(customBaud = 115200) {
    if (!('serial' in navigator)) {
      toast.error('Web Serial tidak tersedia', 'Gunakan Chrome atau Edge versi 89+.');
      return;
    }
    if (connecting) return; // prevent concurrent calls
    connecting = true;
    state.set('connecting');
    errorMsg.set('');

    // Disconnect cleanly if already connected
    if (port) await _disconnect(false);

    try {
      port = await (navigator as any).serial.requestPort();

      // Try baud rates in order
      const bauds = [customBaud, ...[115200, 9600, 38400, 57600, 19200].filter(b => b !== customBaud)];
      let lastErr = '';
      let opened  = false;

      for (const baud of bauds) {
        for (let attempt = 1; attempt <= 2; attempt++) {
          try {
            await openFresh(port, baud);
            baudRate.set(baud);
            opened = true;
            console.log(`[Serial] Opened at ${baud} baud (attempt ${attempt})`);
            break;
          } catch (e: any) {
            lastErr = String(e);
            console.warn(`[Serial] Open ${baud}/${attempt}:`, lastErr);
            if (attempt < 2) await new Promise(r => setTimeout(r, 2500));
          }
        }
        if (opened) break;
      }

      if (!opened) throw new Error(lastErr || 'Gagal membuka port');

      // ── CRITICAL: Start reading the readable stream ────────
      // Bluetooth RFCOMM on macOS requires BOTH readable and writable
      // to be consumed to fully establish the SPP channel.
      // Without this, writes succeed in the browser but never reach the printer.
      startReader();

      // Disconnect listener
      port.addEventListener('disconnect', () => {
        console.log('[Serial] port disconnected event');
        _cleanup();
        toast.warning('Printer terputus');
      });

      const info = port.getInfo?.() ?? {};
      const name = info.usbProductId ? `USB ${info.usbProductId}` : 'Bluetooth Serial Port';
      portName.set(name);

      state.set('connected');
      toast.success('Printer terhubung', `${name} @ ${get(baudRate)} baud`);

    } catch (err: any) {
      const errStr = String(err);
      const cancelled = err?.name === 'NotFoundError' ||
        errStr.toLowerCase().includes('cancel') ||
        errStr.toLowerCase().includes('no port');

      if (cancelled) {
        state.set('disconnected');
        toast.info('Dibatalkan', 'Tidak ada port yang dipilih.');
      } else {
        state.set('error');
        errorMsg.set(errStr);
        toast.error('Gagal konek serial',
          errStr.includes('Failed to open') || errStr.includes('NetworkError')
            ? 'Pastikan printer menyala. Coba baud 9600. Jika gagal terus, restart printer.'
            : errStr
        );
      }
      if (port) { try { await port.close(); } catch { /* ignore */ } }
      port = null;
    } finally {
      connecting = false;
    }
  }

  // ── Background reader: establishes RFCOMM channel ────────
  function startReader() {
    if (!port?.readable) return;
    const r = port.readable.getReader();
    reader = r;
    (async () => {
      try {
        console.log('[Serial] Reader started — RFCOMM channel established');
        while (true) {
          const { done, value } = await r.read();
          if (done) break;
          // Silently discard incoming bytes (status/ACK from printer)
          console.log('[Serial] RX', value?.length, 'bytes (discarded)');
        }
      } catch (e) {
        console.log('[Serial] Reader ended:', e);
      } finally {
        try { r.releaseLock(); } catch { /* ignore */ }
        reader = null;
      }
    })();
  }

  // ── Send raw bytes ────────────────────────────────────────
  async function printRaw(data: Uint8Array): Promise<void> {
    if (!port?.writable) throw new Error('Serial port tidak terhubung');

    console.log('[Serial] printRaw:', data.length, 'bytes');

    // Chunk large payloads to avoid buffer overflow
    const CHUNK = 256;
    for (let i = 0; i < data.length; i += CHUNK) {
      const chunk = data.slice(i, i + CHUNK);
      const w = port.writable.getWriter();
      try {
        await w.write(chunk);
        console.log(`[Serial] wrote chunk [${i}..${i + chunk.length}]`);
      } finally {
        w.releaseLock();
      }
      // Small delay between chunks for Bluetooth flow control
      if (i + CHUNK < data.length) await new Promise(r => setTimeout(r, 10));
    }
    console.log('[Serial] printRaw complete');
  }

  // ── Test print ───────────────────────────────────────────
  async function testPrint(): Promise<void> {
    const enc = new TextEncoder();
    const LF = 0x0a, ESC = 0x1b, GS = 0x1d;
    const bytes = new Uint8Array([
      ESC, 0x40,                                           // init
      ESC, 0x61, 0x01,                                     // center
      ...enc.encode('=== TEST PRINT ==='), LF,
      ...enc.encode('RPP02N Serial OK'), LF,
      ...enc.encode(new Date().toLocaleString('id-ID')), LF,
      ESC, 0x61, 0x00,                                     // left
      LF, LF, LF,                                          // feed
      ESC, 0x69,                                           // cut ESC i
      GS, 0x56, 0x00,                                      // cut GS V 0
      GS, 0x56, 0x41, 0x00,                                // cut GS V 65 0
    ]);
    console.log('[Serial] testPrint', bytes.length, 'bytes');
    await printRaw(bytes);
  }

  // ── Internal disconnect ───────────────────────────────────
  async function _disconnect(notify = true) {
    if (reader) {
      try { await reader.cancel(); } catch { /* ignore */ }
      reader = null;
    }
    if (port) {
      try { await port.close(); } catch { /* ignore */ }
      port = null;
    }
    if (notify) {
      state.set('disconnected');
      portName.set('');
      errorMsg.set('');
    }
  }

  function _cleanup() {
    reader = null;
    port   = null;
    state.set('disconnected');
    portName.set('');
    errorMsg.set('');
  }

  function disconnect()  { _disconnect(true); }
  function forceReset()  { _cleanup(); }

  return {
    state:    { subscribe: state.subscribe },
    portName: { subscribe: portName.subscribe },
    baudRate: { subscribe: baudRate.subscribe },
    errorMsg: { subscribe: errorMsg.subscribe },
    connect,
    disconnect,
    forceReset,
    printRaw,
    testPrint,
    setBaudRate: (b: number) => baudRate.set(b),
  };
}

export const serial = createSerialStore();
export const serialState    = { subscribe: serial.state.subscribe };
export const serialPortName = { subscribe: serial.portName.subscribe };
export const serialBaud     = { subscribe: serial.baudRate.subscribe };
export const serialError    = { subscribe: serial.errorMsg.subscribe };
