<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { bluetooth, btState } from "$lib/stores/bluetooth";
  import { serial, serialState } from "$lib/stores/serial";
  import { bridge, bridgeState, bridgePortName } from "$lib/stores/bridge";
  import { toast } from "$lib/stores/toast";
  import Modal from "$components/ui/Modal.svelte";
  import Spinner from "$components/ui/Spinner.svelte";
  import {
    Printer,
    Ticket,
    FileText,
    Check,
    BluetoothOff,
    Bluetooth,
    Usb,
    Server,
  } from "lucide-svelte";
  import { currency, datetime } from "$lib/utils/format";
  import QRCode from "qrcode";

  export let open = false;
  export let transaction: any = null; // transaction object
  export let items: any[] = []; // transaction items
  export let payments: any[] = []; // payments
  export let tickets: any[] = []; // ticket list from reprint endpoint

  export let venueName = "TiketKu";
  export let venueAddress = "";
  export let venuePhone = "";
  export let venueEmail = "";
  export let venueFooter1 = "Terima kasih!";
  export let venueFooter2 = "";
  export let paperWidth = "80mm";
  export let qrEnabled = true;

  const dispatch = createEventDispatcher<{ close: void }>();

  let printingReceipt = false;
  let printingTickets: Record<string, boolean> = {};
  let printedTickets: Set<string> = new Set();
  let bypassHardwarePrinter = false;

  onMount(() => {
    if (typeof localStorage !== "undefined") {
      bypassHardwarePrinter =
        localStorage.getItem("tk-bypass-printer") === "true";
    }
  });

  $: if (typeof localStorage !== "undefined") {
    localStorage.setItem("tk-bypass-printer", String(bypassHardwarePrinter));
  }

  $: btConnected = $btState === "connected";
  $: serialConnected = $serialState === "connected";
  $: bridgeOnline = $bridgeState === "online";
  $: anyPrinterConnected = bridgeOnline || serialConnected || btConnected;
  $: useHardwarePrinter = anyPrinterConnected && !bypassHardwarePrinter;

  // Character width for printer layout: 32 columns for 58mm, 48 columns for 80mm
  $: charWidth = paperWidth === "58mm" ? 32 : 48;
  // Divider string length in the preview modal UI
  $: dividerCharCount = paperWidth === "58mm" ? 16 : 24;

  // ── Build ESC/POS bytes ──────────────────────────────────
  const ESC = 0x1b;
  const GS = 0x1d;
  const LF = 0x0a;

  function encode(text: string): number[] {
    return Array.from(new TextEncoder().encode(text));
  }

  function line(text = ""): number[] {
    return [...encode(text), LF];
  }

  function center(text: string): number[] {
    return [ESC, 0x61, 0x01, ...encode(text), LF, ESC, 0x61, 0x00];
  }

  function bold(on: boolean): number[] {
    return [ESC, 0x45, on ? 1 : 0];
  }

  function fontSize(size: number): number[] {
    let esc_n = 0x00;
    let gs_n = 0x00;
    if (size === 1) {
      esc_n = 0x10; // double height
      gs_n = 0x01;
    } else if (size === 2) {
      esc_n = 0x20; // double width
      gs_n = 0x10;
    } else if (size === 3) {
      esc_n = 0x30; // double width & height (2x)
      gs_n = 0x11;
    } else if (size === 4) {
      esc_n = 0x00; // 3x width & height
      gs_n = 0x22;
    } else if (size === 5) {
      esc_n = 0x00; // 4x width & height
      gs_n = 0x33;
    } else if (size === 6) {
      esc_n = 0x00; // 5x width & height
      gs_n = 0x44;
    } else if (size === 7) {
      esc_n = 0x00; // 6x width & height
      gs_n = 0x55;
    } else if (size === 8) {
      esc_n = 0x00; // 7x width & height
      gs_n = 0x66;
    } else if (size === 9) {
      esc_n = 0x00; // 8x width & height
      gs_n = 0x77;
    } else if (size === 10) {
      // Restore to double width & height (2x) which is the maximum supported size on portabel printers like RPP02N
      esc_n = 0x30;
      gs_n = 0x11;
    }
    return [ESC, 0x21, esc_n, GS, 0x21, gs_n];
  }

  function divider(char = "-", len = 32): number[] {
    return line(char.repeat(len));
  }

  function cut(): number[] {
    return [LF, LF, LF, ESC, 0x69, GS, 0x56, 0x00, GS, 0x56, 0x41, 0x00];
  }

  // Format a row to align text nicely to left and right columns
  function formatRow(left: string, right: string, width: number): string {
    const space = width - left.length - right.length;
    if (space > 0) {
      return left + " ".repeat(space) + right;
    }
    return left + "\n" + " ".repeat(Math.max(0, width - right.length)) + right;
  }

  // Standard ESC/POS QR code generation for Bluetooth thermal printers
  function buildEscPosQrCode(data: string): number[] {
    const bytes: number[] = [];
    const textBytes = encode(data);
    const len = textBytes.length + 3;
    const pL = len & 0xff;
    const pH = (len >> 8) & 0xff;

    bytes.push(
      // Model selection (Model 2)
      0x1d,
      0x28,
      0x6b,
      0x04,
      0x00,
      0x30,
      0x41,
      0x32,
      0x00,
      // Set module size (size 6)
      0x1d,
      0x28,
      0x6b,
      0x03,
      0x00,
      0x30,
      0x43,
      0x06,
      // Set error correction level L (30)
      0x1d,
      0x28,
      0x6b,
      0x03,
      0x00,
      0x30,
      0x44,
      0x30,
      // Store data in symbol storage
      0x1d,
      0x28,
      0x6b,
      pL,
      pH,
      0x30,
      0x50,
      0x30,
      ...textBytes,
      // Print QR symbol
      0x1d,
      0x28,
      0x6b,
      0x03,
      0x00,
      0x30,
      0x51,
      0x30,
    );
    return bytes;
  }

  function buildReceiptBytes(): Uint8Array {
    const width = charWidth;
    const bytes: number[] = [
      ESC,
      0x40, // init
      ...center(venueName),
    ];
    if (venueAddress) bytes.push(...center(venueAddress));
    if (venuePhone) bytes.push(...center(`Telp: ${venuePhone}`));
    if (venueEmail) bytes.push(...center(`Email: ${venueEmail}`));

    bytes.push(
      ...divider("=", width),
      ...line(`No. Nota : ${transaction?.invoice ?? "-"}`),
      ...line(
        `Tanggal : ${transaction?.created_at ? datetime(transaction.created_at) : "-"}`,
      ),
      ...divider("-", width),
    );
    for (const item of items) {
      bytes.push(...line(item.product_name));
      const qtyPrice = `  x${item.qty}  ${currency(item.unit_price)}`;
      const subtotalStr = currency(item.subtotal);
      bytes.push(...line(formatRow(qtyPrice, subtotalStr, width)));
    }
    bytes.push(
      ...divider("-", width),
      ...line(
        formatRow("Subtotal", currency(transaction?.subtotal ?? 0), width),
      ),
    );
    if ((transaction?.discount ?? 0) > 0) {
      bytes.push(
        ...line(
          formatRow("Diskon", `-${currency(transaction.discount)}`, width),
        ),
      );
    }
    bytes.push(
      ...bold(true),
      ...line(formatRow("TOTAL", currency(transaction?.total ?? 0), width)),
      ...bold(false),
    );
    bytes.push(...divider("-", width));
    for (const p of payments) {
      bytes.push(
        ...line(formatRow(p.method.toUpperCase(), currency(p.amount), width)),
      );
    }
    if ((transaction?.change ?? 0) > 0) {
      bytes.push(
        ...line(formatRow("Kembalian", currency(transaction.change), width)),
      );
    }
    bytes.push(
      ...divider("=", width),
      ...(venueFooter1 ? center(venueFooter1) : []),
      ...(venueFooter2 ? center(venueFooter2) : []),
      ...cut(),
    );
    return new Uint8Array(bytes);
  }

  function buildTicketBytes(ticket: any): Uint8Array {
    const width = charWidth;
    const bytes: number[] = [ESC, 0x40, ...center(venueName)];
    if (venueAddress) bytes.push(...center(venueAddress));
    bytes.push(
      ...divider("-", width),
      ...center(
        `( ${ticket.product_name ?? "Tiket"}${ticket.variant_name ? ` - ${ticket.variant_name}` : ""} )`,
      ),
      ...divider("-", width),
      LF,
      ...center("NOMOR URUT"),
      ...fontSize(10),
      ...bold(true),
      ...center(String(ticket.queue_number ?? 1)),
      ...bold(false),
      ...fontSize(0),
      LF,
    );

    if (qrEnabled) {
      bytes.push(
        ESC,
        0x61,
        0x01, // Center align for QR code
        ...buildEscPosQrCode(ticket.code),
        LF,
        ESC,
        0x61,
        0x00, // Reset to left align
      );
    }

    bytes.push(
      ...divider("-", width),
      ...center(
        `Tanggal: ${transaction?.created_at ? datetime(transaction.created_at) : "-"}`,
      ),
      ...divider("-", width),
      ...center("Tunjukkan ke petugas"),
      ...center(`No. Transaksi: ${transaction?.invoice ?? "-"}`),
      ...cut(),
    );
    return new Uint8Array(bytes);
  }

  async function sendToPrinter(data: Uint8Array): Promise<void> {
    if (btConnected) {
      await bluetooth.printRaw(data);
    } else if (serialConnected) {
      await serial.printRaw(data);
    } else if (bridgeOnline) {
      await bridge.printRaw(data);
    } else {
      throw new Error("Tidak ada printer yang terhubung");
    }
  }

  function printViaWindow(content: string, title: string) {
    const isTicket = title.startsWith("Tiket");
    const win = window.open(
      "",
      "_blank",
      isTicket ? "width=380,height=700" : "width=450,height=600",
    );
    if (!win) {
      toast.error("Pop-up diblokir", "Izinkan pop-up di browser");
      return;
    }

    const paperWidthCSS = paperWidth === "58mm" ? "58mm" : "80mm";
    const bodyWidthCSS = isTicket
      ? paperWidth === "58mm"
        ? "200px"
        : "280px"
      : paperWidth === "58mm"
        ? "220px"
        : "302px";

    win.document.write(`
      <!doctype html>
      <html>
      <head>
        <title>${title}</title>
        <style>
          *, *::before, *::after { box-sizing: border-box; }
          body {
            font-family: 'Courier New', monospace;
            font-size: 11px;
            line-height: 1.5;
            padding: 12px 10px;
            width: ${bodyWidthCSS};
            margin: 0 auto;
            color: #000;
            background: #fff;
          }
          p { margin: 0; }
          .center { text-align: center; }
          .bold   { font-weight: bold; }
          .divider { border-top: 1px dashed #000; margin: 4px 0; }
          .row  { display: flex; justify-content: space-between; }
          .total { font-size: 13px; font-weight: bold; border-top: 1px solid #000; padding-top: 4px; margin-top: 4px; }
          .ticket-code { font-size: 16px; font-weight: bold; text-align: center; letter-spacing: 0.1em; margin: 8px 0; }
          @media print {
            @page { margin: 0; size: ${paperWidthCSS} auto; }
            body  { width: ${paperWidthCSS}; padding: 6px 8px; }
          }
        </style>
      </head>
      <body>${content}</body>
      </html>
    `);
    win.document.close();
    setTimeout(() => {
      win.print();
      win.close();
    }, 300);
  }

  async function printReceipt() {
    printingReceipt = true;
    try {
      if (useHardwarePrinter) {
        await sendToPrinter(buildReceiptBytes());
        const method = bridgeOnline
          ? "Bridge"
          : serialConnected
            ? "Serial"
            : "Bluetooth";
        toast.success("Nota dikirim ke printer", method);
      } else {
        const itemRows = items
          .map(
            (i) =>
              `<div class="row"><span>${i.product_name} x${i.qty}</span><span>${currency(i.subtotal)}</span></div>`,
          )
          .join("");
        const paymentRows = payments
          .map(
            (p) =>
              `<div class="row"><span>${p.method.toUpperCase()}</span><span>${currency(p.amount)}</span></div>`,
          )
          .join("");
        const html = `
          <p class="center bold" style="font-size: 13px; margin-bottom: 2px;">${venueName}</p>
          ${venueAddress ? `<p class="center">${venueAddress}</p>` : ""}
          ${venuePhone ? `<p class="center">Telp: ${venuePhone}</p>` : ""}
          ${venueEmail ? `<p class="center">Email: ${venueEmail}</p>` : ""}
          <div class="divider"></div>
          <p>No. Nota : ${transaction?.invoice ?? "-"}</p>
          <p>Tanggal : ${transaction?.created_at ? datetime(transaction.created_at) : "-"}</p>
          <div class="divider"></div>
          ${itemRows}
          <div class="divider"></div>
          <div class="row"><span>Subtotal</span><span>${currency(transaction?.subtotal ?? 0)}</span></div>
          ${(transaction?.discount ?? 0) > 0 ? `<div class="row" style="color:red"><span>Diskon</span><span>-${currency(transaction.discount)}</span></div>` : ""}
          <div class="row total"><span>TOTAL</span><span>${currency(transaction?.total ?? 0)}</span></div>
          ${paymentRows}
          ${(transaction?.change ?? 0) > 0 ? `<div class="row"><span>Kembalian</span><span>${currency(transaction.change)}</span></div>` : ""}
          <div class="divider"></div>
          <p class="center">${venueFooter1 ? venueFooter1 : ""}</p>
          ${venueFooter2 ? `<p class="center">${venueFooter2}</p>` : ""}
        `;
        printViaWindow(html, `Nota ${transaction?.invoice}`);
      }
    } catch (e: any) {
      toast.error("Gagal cetak nota", e.message);
    } finally {
      printingReceipt = false;
    }
  }

  async function printOneTicket(ticket: any) {
    printingTickets = { ...printingTickets, [ticket.id]: true };
    try {
      if (useHardwarePrinter) {
        await sendToPrinter(buildTicketBytes(ticket));
        toast.success("Tiket dikirim ke printer", ticket.code);
      } else {
        let qrImgHtml = "";
        if (qrEnabled) {
          try {
            const qrDataUrl = await QRCode.toDataURL(ticket.code, {
              width: 160,
              margin: 1,
            });
            qrImgHtml = `<div class="center" style="margin: 10px 0;"><img src="${qrDataUrl}" style="width: 140px; height: 140px;" /></div>`;
          } catch (err) {
            console.error("Gagal generate QR Code:", err);
          }
        }

        const html = `
          <!-- Header venue -->
          <div style="text-align:center; margin-bottom:8px;">
            <div style="font-size:14px; font-weight:bold;">${venueName}</div>
            ${venueAddress ? `<div style="font-size:10px; color:#555; margin-top:2px;">${venueAddress}</div>` : ""}
          </div>

          <div style="border-top:1px dashed #000; margin:8px 0;"></div>

          <!-- Product info / Wahana -->
          <div style="text-align:center; font-weight:bold; font-size:12px; padding:2px 0;">
            ( ${ticket.product_name ?? "Tiket"}${ticket.variant_name ? ` - ${ticket.variant_name}` : ""} )
          </div>

          <div style="border-top:1px dashed #000; margin:8px 0;"></div>

          <!-- Queue number large (SVG responsive to prevent page downscaling) -->
          <div style="text-align:center; margin:5px 0; width:100%; display:flex; justify-content:center; align-items:center;">
            <div style="width: 100%; max-width: 260px;">
              <div style="font-size:11px; font-weight:bold; text-transform:uppercase; color:#555; margin-bottom:4px;">Nomor Urut</div>
              <svg viewBox="0 0 200 120" style="width:100%; height:auto; display:block; margin:0 auto;">
                <text x="100" y="95" text-anchor="middle" font-size="110" font-weight="900" font-family="sans-serif" fill="#000" letter-spacing="-2">${ticket.queue_number ?? 1}</text>
              </svg>
            </div>
          </div>

          <!-- QR Code -->
          ${qrImgHtml}

          <div style="border-top:1px dashed #000; margin:8px 0;"></div>

          <!-- Date -->
          <div style="text-align:center; font-size:10px;">
            Tanggal: ${transaction?.created_at ? datetime(transaction.created_at) : "-"}
          </div>

          <div style="border-top:1px dashed #000; margin:8px 0;"></div>

          <!-- Footer instruction -->
          <div style="text-align:center; font-size:10px; font-weight:bold; text-transform:uppercase;">
            Tunjukan Ke Petugas
          </div>
          <div style="text-align:center; font-size:10px; font-family:monospace; margin-top:4px;">
            No. Transaksi: ${transaction?.invoice ?? "-"}
          </div>
        `;
        printViaWindow(html, `Tiket ${ticket.code}`);
      }
      printedTickets = new Set([...printedTickets, ticket.id]);
    } catch (e: any) {
      toast.error("Gagal cetak tiket", e.message);
    } finally {
      printingTickets = { ...printingTickets, [ticket.id]: false };
    }
  }

  async function printAllTickets() {
    for (const ticket of tickets) {
      await printOneTicket(ticket);
      await new Promise((r) => setTimeout(r, 500));
    }
  }
</script>

<Modal
  bind:open
  title="Cetak Dokumen"
  size="md"
  on:close={() => dispatch("close")}
>
  <div class="modal-body">
    <!-- Printer status -->
    {#if btConnected}
      <div class="bt-ok">
        <div class="dot dot-success dot-pulse"></div>
        <Bluetooth size={14} />
        <span>Printer Bluetooth BLE terhubung & siap ({paperWidth})</span>
      </div>
    {:else if serialConnected}
      <div class="bt-ok">
        <div class="dot dot-success dot-pulse"></div>
        <Usb size={14} />
        <span>Printer Serial terhubung & siap ({paperWidth})</span>
      </div>
    {:else if bridgeOnline}
      <div class="bt-ok">
        <div class="dot dot-success dot-pulse"></div>
        <Server size={14} />
        <span
          >Print Bridge online ({$bridgePortName || "RPP02N"}) ({paperWidth})</span
        >
      </div>
    {:else}
      <div class="bt-warning">
        <BluetoothOff size={16} />
        <span
          >Tidak ada printer terhubung — akan menggunakan cetak browser
          (pop-up).</span
        >
      </div>
    {/if}

    <div
      style="margin-bottom: 16px; display: flex; align-items: center; justify-content: space-between; background: var(--bg-subtle); padding: 10px 14px; border-radius: var(--r-md); border: 1px solid var(--border);"
    >
      <div style="display: flex; align-items: center; gap: 8px;">
        <input
          type="checkbox"
          id="bypass-printer"
          bind:checked={bypassHardwarePrinter}
          style="width: 16px; height: 16px; cursor: pointer;"
        />
        <label
          for="bypass-printer"
          style="font-size: 0.875rem; font-weight: 500; cursor: pointer; color: var(--text-1);"
          >Bypass Printer Fisik (Gunakan Cetak Browser)</label
        >
      </div>
      <span style="font-size: 0.75rem; color: var(--text-3);">Pop-up PDF</span>
    </div>

    <!-- NOTA Section -->
    <div class="print-section">
      <div class="section-label">
        <FileText size={16} style="color:var(--brand-500);" />
        <span>Nota / Struk Pembayaran ({paperWidth})</span>
      </div>

      <div class="nota-preview-container">
        <div
          class="nota-preview"
          class:paper-58mm={paperWidth === "58mm"}
          class:paper-80mm={paperWidth === "80mm"}
        >
          <div class="preview-header">{venueName}</div>
          {#if venueAddress}<div class="preview-sub">{venueAddress}</div>{/if}
          {#if venuePhone}<div class="preview-sub">Telp: {venuePhone}</div>{/if}
          {#if venueEmail}<div class="preview-sub">
              Email: {venueEmail}
            </div>{/if}
          <div class="preview-divider">{"- ".repeat(dividerCharCount)}</div>
          <div class="preview-row">
            <span>No. Nota</span><span>{transaction?.invoice ?? "-"}</span>
          </div>
          <div class="preview-row">
            <span>Tanggal</span><span style="font-size: 9px;"
              >{transaction?.created_at
                ? datetime(transaction.created_at)
                : "-"}</span
            >
          </div>
          <div class="preview-divider">{"- ".repeat(dividerCharCount)}</div>
          {#each items as item}
            <div class="preview-row sm-name">
              <span>{item.product_name}</span>
            </div>
            <div class="preview-row sm-details">
              <span>x{item.qty} {currency(item.unit_price)}</span>
              <span>{currency(item.subtotal)}</span>
            </div>
          {/each}
          <div class="preview-divider">{"- ".repeat(dividerCharCount)}</div>
          <div class="preview-row">
            <span>Subtotal</span><span
              >{currency(transaction?.subtotal ?? 0)}</span
            >
          </div>
          {#if (transaction?.discount ?? 0) > 0}
            <div class="preview-row" style="color:var(--rose);">
              <span>Diskon</span><span>-{currency(transaction.discount)}</span>
            </div>
          {/if}
          <div class="preview-row bold">
            <span>TOTAL</span><span>{currency(transaction?.total ?? 0)}</span>
          </div>
          {#each payments as p}
            <div class="preview-row" style="color:#666;">
              <span>{p.method.toUpperCase()}</span><span
                >{currency(p.amount)}</span
              >
            </div>
          {/each}
          {#if (transaction?.change ?? 0) > 0}
            <div class="preview-row" style="color:var(--emerald);">
              <span>Kembalian</span><span>{currency(transaction.change)}</span>
            </div>
          {/if}
          <div class="preview-divider">{"- ".repeat(dividerCharCount)}</div>
          {#if venueFooter1}<div class="preview-footer">
              {venueFooter1}
            </div>{/if}
          {#if venueFooter2}<div class="preview-footer">
              {venueFooter2}
            </div>{/if}
        </div>
      </div>

      <button
        class="btn btn-primary"
        style="gap:6px;width:100%;"
        on:click={printReceipt}
        disabled={printingReceipt}
      >
        {#if printingReceipt}<Spinner size={14} color="#fff" />{:else}<Printer
            size={14}
          />{/if}
        Cetak Nota
      </button>
    </div>

    <!-- TIKET Section -->
    {#if tickets.length > 0}
      <div class="print-section">
        <div class="section-label">
          <Ticket size={16} style="color:var(--emerald);" />
          <span>Tiket ({tickets.length} tiket)</span>
          {#if tickets.length > 1}
            <button
              class="btn btn-secondary btn-sm"
              style="margin-left:auto;gap:5px;"
              on:click={printAllTickets}
            >
              <Printer size={12} /> Cetak Semua
            </button>
          {/if}
        </div>

        <div class="ticket-list">
          {#each tickets as ticket (ticket.id)}
            <div
              class="ticket-item"
              class:printed={printedTickets.has(ticket.id)}
            >
              <div class="ti-info">
                <div style="display: flex; align-items: center; gap: 8px;">
                  <span class="ti-code mono">{ticket.code}</span>
                  <span
                    style="font-size: 0.75rem; font-weight: bold; background: var(--brand-50); color: var(--brand-600); padding: 1px 6px; border-radius: var(--r-full);"
                    >Urut: #{ticket.queue_number ?? 1}</span
                  >
                </div>
                <span class="ti-name"
                  >{ticket.product_name ?? "Tiket"}{ticket.variant_name
                    ? ` \u2014 ${ticket.variant_name}`
                    : ""}</span
                >
              </div>
              <div style="display:flex;align-items:center;gap:6px;">
                {#if printedTickets.has(ticket.id)}
                  <span class="printed-badge"><Check size={11} /> Tercetak</span
                  >
                {/if}
                <button
                  class="btn btn-sm"
                  style="gap:5px;"
                  class:btn-secondary={printedTickets.has(ticket.id)}
                  class:btn-success={!printedTickets.has(ticket.id)}
                  on:click={() => printOneTicket(ticket)}
                  disabled={printingTickets[ticket.id]}
                >
                  {#if printingTickets[ticket.id]}
                    <Spinner size={12} color="#fff" />
                  {:else}
                    <Printer size={12} />
                  {/if}
                  {printedTickets.has(ticket.id) ? "Cetak Ulang" : "Cetak"}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="no-tickets">
        <Ticket size={20} style="color:var(--text-3);" />
        <span>Tidak ada tiket untuk transaksi ini</span>
      </div>
    {/if}
  </div>

  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => dispatch("close")}
      >Tutup</button
    >
  </div>
</Modal>

<style>
  .bt-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: var(--r-md);
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.25);
    color: var(--amber);
    font-size: 0.875rem;
    margin-bottom: 16px;
  }
  .bt-ok {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: var(--r-md);
    background: rgba(16, 185, 129, 0.06);
    border: 1px solid rgba(16, 185, 129, 0.2);
    color: var(--emerald);
    font-size: 0.875rem;
    margin-bottom: 16px;
  }
  .print-section {
    background: var(--bg-subtle);
    border-radius: var(--r-lg);
    padding: 16px;
    margin-bottom: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .section-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-1);
  }

  /* Nota preview container */
  .nota-preview-container {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 16px;
    display: flex;
    justify-content: center;
    overflow-x: auto;
  }

  /* Nota preview */
  .nota-preview {
    background: #fff;
    border: 1px dashed #ccc;
    border-radius: 4px;
    padding: 14px 10px;
    font-family: "Courier New", monospace;
    color: #000;
    line-height: 1.5;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
    transition: width var(--ease-base);
  }
  [data-theme="dark"] .nota-preview {
    background: #1a1a1a;
    color: #eee;
    border-color: #444;
  }

  .nota-preview.paper-58mm {
    width: 220px;
    font-size: 10px;
  }
  .nota-preview.paper-80mm {
    width: 302px;
    font-size: 11px;
  }

  .preview-header {
    font-weight: bold;
    text-align: center;
    font-size: 1.15em;
    margin-bottom: 2px;
  }
  .preview-sub {
    text-align: center;
    font-size: 0.9em;
    color: #555;
  }
  [data-theme="dark"] .preview-sub {
    color: #aaa;
  }

  .preview-divider {
    color: #999;
    margin: 6px 0;
    letter-spacing: -2px;
    text-align: center;
  }
  .preview-row {
    display: flex;
    justify-content: space-between;
    padding: 2px 0;
  }
  .preview-row.sm-name {
    font-weight: 600;
    padding-bottom: 0;
  }
  .preview-row.sm-details {
    font-size: 0.95em;
    color: #555;
    padding-left: 8px;
    padding-top: 0;
  }
  :global([data-theme="dark"]) .preview-row.sm-details {
    color: #aaa;
  }

  .preview-row.bold {
    font-weight: bold;
    border-top: 1px solid #ccc;
    padding-top: 4px;
    margin-top: 4px;
    font-size: 1.1em;
  }
  .preview-footer {
    text-align: center;
    font-size: 0.95em;
    font-weight: 500;
    margin-top: 2px;
  }

  /* Tickets */
  .ticket-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ticket-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: var(--r-md);
    background: var(--bg-surface);
    border: 1px solid var(--border);
    gap: 10px;
    flex-wrap: wrap;
    transition: opacity var(--ease-fast);
  }
  .ticket-item.printed {
    opacity: 0.7;
  }
  .ti-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .ti-code {
    font-size: 0.875rem;
    font-weight: 700;
    color: var(--text-1);
  }
  .ti-name {
    font-size: 0.8125rem;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .printed-badge {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 0.75rem;
    color: var(--emerald);
    background: rgba(16, 185, 129, 0.1);
    padding: 2px 6px;
    border-radius: var(--r-full);
  }
  .no-tickets {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px;
    color: var(--text-3);
    font-size: 0.875rem;
    background: var(--bg-subtle);
    border-radius: var(--r-md);
  }
</style>
