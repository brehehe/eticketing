/**
 * Print Bridge Store
 * 
 * Mengirim data ESC/POS ke http://localhost:9100 (printbridge.js)
 * yang menulis langsung ke serial port Bluetooth di level OS.
 */
import { writable, get } from 'svelte/store';
import { toast } from './toast';

export type BridgeState = 'unknown' | 'online' | 'offline';

const BRIDGE_URL = 'http://127.0.0.1:9100';

function createBridgeStore() {
  const state   = writable<BridgeState>('unknown');
  const portName = writable<string>('');

  // Check if bridge is running
  async function checkStatus(): Promise<boolean> {
    try {
      const res = await fetch(`${BRIDGE_URL}/status`, { signal: AbortSignal.timeout(2000) });
      const json = await res.json();
      state.set('online');
      portName.set(json.port ?? '');
      return json.portExists ?? true;
    } catch {
      state.set('offline');
      return false;
    }
  }

  // Send ESC/POS bytes to the bridge
  async function printRaw(data: Uint8Array): Promise<void> {
    // Convert to base64
    const base64 = btoa(String.fromCharCode(...Array.from(data)));
    const res = await fetch(`${BRIDGE_URL}/print`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ data: base64 }),
      signal:  AbortSignal.timeout(10000),
    });
    if (!res.ok) {
      const json = await res.json().catch(() => ({}));
      throw new Error(json.error ?? `Bridge error: HTTP ${res.status}`);
    }
  }

  // Quick test print
  async function testPrint(): Promise<void> {
    const enc = new TextEncoder();
    const LF = 0x0a, ESC = 0x1b, GS = 0x1d;
    const bytes = new Uint8Array([
      ESC, 0x40,
      ESC, 0x61, 0x01,
      ...enc.encode('=== TEST PRINT ==='), LF,
      ...enc.encode('Print Bridge OK'), LF,
      ...enc.encode(new Date().toLocaleString('id-ID')), LF,
      ESC, 0x61, 0x00,
      LF, LF, LF,
      ESC, 0x69,
      GS, 0x56, 0x00,
      GS, 0x56, 0x41, 0x00,
    ]);
    await printRaw(bytes);
  }

  return {
    state:    { subscribe: state.subscribe },
    portName: { subscribe: portName.subscribe },
    checkStatus,
    printRaw,
    testPrint,
    get isOnline() { return get(state) === 'online'; },
  };
}

export const bridge = createBridgeStore();
export const bridgeState = { subscribe: bridge.state.subscribe };
export const bridgePortName = { subscribe: bridge.portName.subscribe };
