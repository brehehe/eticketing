#!/usr/bin/env node
/**
 * TiketKu Print Bridge
 * 
 * Menerima data ESC/POS dari browser via HTTP POST,
 * lalu menulis langsung ke serial port Bluetooth (RPP02N).
 * 
 * Jalankan: node printbridge.js
 * Port: http://localhost:9100
 */

const http = require('http');
const fs   = require('fs');
const path = require('path');

const BRIDGE_PORT  = 9100;
const SERIAL_PORT  = process.env.PRINTER_PORT || '/dev/cu.RPP02N';

// ── CORS helper ───────────────────────────────────────────────
function cors(res) {
  res.setHeader('Access-Control-Allow-Origin',  '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
}

function writeToSerial(bytes) {
  return new Promise((resolve, reject) => {
    // Check port exists
    if (!fs.existsSync(SERIAL_PORT)) {
      return reject(new Error(`Port ${SERIAL_PORT} tidak ditemukan. Pastikan RPP02N sudah di-pair via System Settings → Bluetooth.`));
    }

    // Configure baudrate via stty (helps establish correct line settings)
    try {
      const baud = process.env.PRINTER_BAUD || '9600';
      const { execSync } = require('child_process');
      execSync(`stty -f ${SERIAL_PORT} ${baud} cs8 -parenb -cstopb raw -echo`);
      console.log(`[Bridge] Port speed set to ${baud} baud`);
    } catch (e) {
      console.warn(`[Bridge] Gagal set port settings via stty:`, e.message);
    }

    // Open port, write, wait, close
    fs.open(SERIAL_PORT, 'r+', (err, fd) => {
      if (err) return reject(new Error(`Gagal buka port: ${err.message}`));
      fs.write(fd, bytes, (werr, written) => {
        if (werr) {
          fs.close(fd, () => {});
          return reject(new Error(`Gagal tulis: ${werr.message}`));
        }
        
        // CRITICAL: Wait 1500ms before closing the port descriptor to allow
        // the operating system to flush the buffer over the slow Bluetooth RFCOMM link.
        setTimeout(() => {
          fs.close(fd, (cerr) => {
            if (cerr) return reject(new Error(`Gagal close port: ${cerr.message}`));
            resolve(written);
          });
        }, 1500);
      });
    });
  });
}

// ── HTTP server ───────────────────────────────────────────────
const server = http.createServer(async (req, res) => {
  cors(res);

  // OPTIONS preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(204); res.end(); return;
  }

  // GET /status  — health check
  if (req.method === 'GET' && req.url === '/status') {
    const portExists = fs.existsSync(SERIAL_PORT);
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      ok:       true,
      port:     SERIAL_PORT,
      portExists,
      message:  portExists ? 'Print bridge siap' : `Port ${SERIAL_PORT} tidak ditemukan — pastikan printer di-pair`,
    }));
    return;
  }

  // POST /print  — receive base64 ESC/POS bytes and send to printer
  if (req.method === 'POST' && req.url === '/print') {
    let body = '';
    req.on('data', chunk => (body += chunk));
    req.on('end', async () => {
      try {
        const { data } = JSON.parse(body);
        if (!data) throw new Error('Field "data" (base64) diperlukan');
        const bytes = Buffer.from(data, 'base64');

        console.log(`[Bridge] Mencetak ${bytes.length} bytes ke ${SERIAL_PORT}...`);
        const written = await writeToSerial(bytes);
        console.log(`[Bridge] OK — ${written} bytes tertulis`);

        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: true, bytes: written }));
      } catch (e) {
        console.error('[Bridge] Error:', e.message);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: false, error: e.message }));
      }
    });
    return;
  }

  res.writeHead(404); res.end();
});

server.listen(BRIDGE_PORT, '0.0.0.0', () => {
  console.log('');
  console.log('╔══════════════════════════════════════╗');
  console.log('║   TiketKu Print Bridge — Siap!       ║');
  console.log(`║   http://127.0.0.1:${BRIDGE_PORT}              ║`);
  console.log(`║   Port: ${SERIAL_PORT}           ║`);
  console.log('╚══════════════════════════════════════╝');
  console.log('');
  console.log('Gunakan PRINTER_PORT=... untuk ganti port.');
  console.log('Contoh: PRINTER_PORT=/dev/cu.RPP02N node printbridge.js');
  console.log('');
});

server.on('error', (e) => {
  if (e.code === 'EADDRINUSE') {
    console.error(`[Bridge] Port ${BRIDGE_PORT} sudah dipakai. Matikan proses lain dulu.`);
  } else {
    console.error('[Bridge] Server error:', e.message);
  }
  process.exit(1);
});
