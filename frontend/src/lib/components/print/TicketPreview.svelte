<script lang="ts">
  import type { Ticket, VenueSettings } from '$lib/types';
  import { date } from '$lib/utils/format';
  import QRCode from 'qrcode';
  import { onMount } from 'svelte';

  export let ticket: Ticket;
  export let venue: VenueSettings;

  $: ticketAny = ticket as any;

  let qrCanvas: HTMLCanvasElement;

  onMount(async () => {
    if (qrCanvas && ticket.qr_data) {
      await QRCode.toCanvas(qrCanvas, ticket.qr_data, {
        width: 120,
        margin: 1,
        color: { dark: '#000000', light: '#ffffff' }
      });
    }
  });
</script>

<div class="ticket" role="document" aria-label="Preview tiket">
  <!-- Header -->
  <div class="ticket-header">
    {#if venue.logo}
      <img src={venue.logo} alt={venue.name} class="ticket-logo" />
    {:else}
      <div class="ticket-logo-placeholder">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" aria-hidden="true">
          <rect width="32" height="32" rx="8" fill="#6366f1"/>
          <path d="M8 16h16M16 8l8 8-8 8" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    {/if}
    <span class="ticket-venue-name">{venue.name}</span>
  </div>

  <div class="ticket-divider"></div>

  <!-- Product -->
  <div class="ticket-product">
    <span class="ticket-product-name">
      {ticket.product?.name ?? 'Tiket'}
      {#if ticketAny.variant}
        — {ticketAny.variant.name ?? ''}
      {/if}
    </span>
  </div>

  <!-- QR Code -->
  <div class="ticket-qr">
    <canvas bind:this={qrCanvas} aria-label="QR Code tiket {ticket.code}"></canvas>
  </div>

  <!-- Code -->
  <div class="ticket-code">{ticket.code}</div>

  {#if ticket.valid_until}
    <div class="ticket-valid">
      Valid: {date(ticket.valid_until)}
    </div>
  {:else}
    <div class="ticket-valid">Single Use</div>
  {/if}

  <div class="ticket-divider"></div>

  <div class="ticket-footer">
    Tunjukkan QR ini ke petugas
  </div>
</div>

<style>
  .ticket {
    width: 302px;
    background: #fff;
    color: #000;
    font-family: 'Inter', sans-serif;
    padding: 16px 12px;
    border: 1px dashed #ccc;
    border-radius: 8px;
    text-align: center;
  }
  .ticket-header {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
    margin-bottom: 10px;
  }
  .ticket-logo { width: 32px; height: 32px; border-radius: 6px; }
  .ticket-logo-placeholder { flex-shrink: 0; }
  .ticket-venue-name { font-size: 13px; font-weight: 700; }
  .ticket-divider { height: 1px; background: #ddd; margin: 8px 0; }
  .ticket-product { margin: 8px 0; }
  .ticket-product-name { font-size: 15px; font-weight: 700; }
  .ticket-qr {
    display: flex;
    justify-content: center;
    margin: 12px 0;
  }
  .ticket-qr canvas { border-radius: 4px; }
  .ticket-code {
    font-family: 'Courier New', monospace;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.1em;
    margin-bottom: 4px;
  }
  .ticket-valid { font-size: 11px; color: #666; margin-bottom: 4px; }
  .ticket-footer { font-size: 11px; color: #888; margin-top: 4px; }
</style>
