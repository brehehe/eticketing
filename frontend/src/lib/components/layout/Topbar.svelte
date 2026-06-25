<script lang="ts">
  import { page } from '$app/stores';
  import { auth } from '$lib/stores/auth';
  import { btState, btPrintQueue } from '$lib/stores/bluetooth';
  import Avatar from '$components/ui/Avatar.svelte';
  import Badge from '$components/ui/Badge.svelte';
  import {
    Menu, Bell, Printer, Bluetooth, BluetoothOff,
    Search, Plus, ChevronRight
  } from 'lucide-svelte';

  export let onMenuToggle: () => void = () => {};

  $: btConnected = $btState === 'connected';
  $: queueCount = $btPrintQueue.filter(j => j.status === 'queued' || j.status === 'printing').length;

  // Breadcrumb from URL
  const routeLabels: Record<string, string> = {
    dashboard: 'Dashboard',
    kasir: 'Kasir',
    tiket: 'Tiket',
    produk: 'Produk',
    users: 'Pengguna',
    roles: 'Role & Akses',
    laporan: 'Laporan',
    pengaturan: 'Pengaturan',
    printer: 'Printer',
  };

  $: segments = $page.url.pathname
    .split('/')
    .filter(Boolean)
    .map(s => ({ slug: s, label: routeLabels[s] ?? s }));
</script>

<header class="topbar" role="banner">
  <!-- Left: menu + breadcrumb -->
  <div class="topbar-left">
    <button
      class="btn btn-ghost btn-icon menu-btn"
      on:click={onMenuToggle}
      aria-label="Toggle menu"
    >
      <Menu size={20} />
    </button>

    <nav class="breadcrumb" aria-label="Breadcrumb">
      {#each segments as seg, i}
        {#if i > 0}
          <ChevronRight size={12} class="bc-sep" aria-hidden="true" />
        {/if}
        <span
          class="bc-item"
          class:bc-current={i === segments.length - 1}
          aria-current={i === segments.length - 1 ? 'page' : undefined}
        >
          {seg.label}
        </span>
      {/each}
    </nav>
  </div>

  <!-- Right: status indicators + user -->
  <div class="topbar-right">
    <!-- Bluetooth status -->
    <button
      class="status-btn"
      class:connected={btConnected}
      on:click={() => window.location.href = '/printer'}
      title="Status printer bluetooth"
      aria-label="Status printer: {$btState}"
    >
      {#if btConnected}
        <Bluetooth size={16} />
      {:else}
        <BluetoothOff size={16} />
      {/if}
      {#if queueCount > 0}
        <span class="status-count">{queueCount}</span>
      {/if}
    </button>

    <!-- Print queue -->
    {#if queueCount > 0}
      <button
        class="status-btn"
        title="Antrian cetak: {queueCount}"
        aria-label="{queueCount} item dalam antrian cetak"
      >
        <Printer size={16} />
        <span class="status-count">{queueCount}</span>
      </button>
    {/if}

    <!-- Notifications -->
    <button class="status-btn" aria-label="Notifikasi">
      <Bell size={16} />
    </button>

    <div class="topbar-divider" aria-hidden="true"></div>

    <!-- User -->
    {#if $auth}
      <button class="user-chip" aria-label="Menu pengguna">
        <Avatar name={$auth.user.name} size="xs" />
        <span class="user-chip-name">{$auth.user.name}</span>
      </button>
    {/if}
  </div>
</header>

<style>
  .topbar {
    height: var(--topbar-h);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 40;
    gap: 12px;
  }

  .topbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .menu-btn { color: var(--text-2); }
  .menu-btn:hover { color: var(--text-1); }

  /* Breadcrumb */
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.875rem;
    color: var(--text-2);
    overflow: hidden;
  }
  :global(.bc-sep) { color: var(--text-3); flex-shrink: 0; }
  .bc-item {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-transform: capitalize;
  }
  .bc-current {
    color: var(--text-1);
    font-weight: 600;
  }

  /* Right side */
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .status-btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--r-md);
    color: var(--text-3);
    cursor: pointer;
    transition: background var(--ease-fast), color var(--ease-fast);
    border: none;
    background: none;
    font-family: var(--font-sans);
  }
  .status-btn:hover {
    background: var(--bg-muted);
    color: var(--text-1);
  }
  .status-btn.connected { color: var(--emerald); }

  .status-count {
    position: absolute;
    top: 3px;
    right: 3px;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    background: var(--brand-500);
    color: #fff;
    font-size: 0.5625rem;
    font-weight: 700;
    border-radius: var(--r-full);
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .topbar-divider {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 4px;
  }

  /* User chip */
  .user-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px 0 4px;
    border-radius: var(--r-full);
    background: none;
    border: none;
    cursor: pointer;
    transition: background var(--ease-fast);
    font-family: var(--font-sans);
  }
  .user-chip:hover { background: var(--bg-muted); }
  .user-chip-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-1);
    white-space: nowrap;
  }

  /* Hide name on small screens */
  @media (max-width: 640px) {
    .user-chip-name { display: none; }
    .topbar { padding: 0 12px; }
  }
</style>
