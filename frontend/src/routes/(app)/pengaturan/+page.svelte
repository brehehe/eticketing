<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { toast } from '$lib/stores/toast';
  import { theme } from '$lib/stores/theme';
  import { bluetooth, btState as btStateStore } from '$lib/stores/bluetooth';
  import { settingsApi } from '$lib/api/resources';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import {
    Building2, Ticket, Printer, Settings, Database,
    Bluetooth, BluetoothOff, Save, RefreshCw, HardDrive
  } from 'lucide-svelte';

  let activeTab = 'venue';
  let saving = false;
  let loading = true;

  $: btState     = $btStateStore;
  $: btConnected = btState === 'connected';

  function changeTheme(e: Event) {
    const select = e.currentTarget as HTMLSelectElement;
    theme.set(select.value as any);
  }

  let settings = {
    venue:   { name:'', address:'', phone:'', email:'', operating_hours:'', footer_line_1:'Terima kasih!', footer_line_2:'TiketKu POS & Ticketing' },
    ticket:  { print_enabled:true, qr_enabled:true, expiry_hours:24, ticket_prefix:'TK' },
    printer: { auto_retry:true, retry_count:3, paper_width:'80mm' },
    shift:   { shift_enabled:false, auto_closing:false, opening_balance:0 },
    system:  { session_timeout_min:480, cache_ttl_sec:300 },
  };

  onMount(async () => {
    try {
      const res = await settingsApi.get();
      const d = (res as any)?.data;
      if (d) {
        if (d.venue)   settings.venue   = { ...settings.venue,   ...d.venue   };
        if (d.ticket)  settings.ticket  = { ...settings.ticket,  ...d.ticket  };
        if (d.printer) settings.printer = { ...settings.printer, ...d.printer };
        if (d.shift)   settings.shift   = { ...settings.shift,   ...d.shift   };
        if (d.system)  settings.system  = { ...settings.system,  ...d.system  };
      }
    } catch {
      // settings remain at defaults
    } finally {
      loading = false;
    }
  });

  async function saveSettings() {
    saving = true;
    try {
      await settingsApi.update(settings);
      toast.success('Pengaturan tersimpan');
    } catch (e: any) {
      toast.error('Gagal menyimpan', e.message);
    } finally {
      saving = false;
    }
  }

  const tabs = [
    { id:'venue',   label:'Venue',   icon:Building2 },
    { id:'ticket',  label:'Tiket',   icon:Ticket    },
    { id:'printer', label:'Printer', icon:Printer   },
    { id:'shift',   label:'Kasir',   icon:Settings  },
    { id:'system',  label:'Sistem',  icon:Database  },
  ];
</script>

<svelte:head><title>Pengaturan \u2014 TiketKu</title></svelte:head>

<div class="settings-layout">
  <nav class="settings-nav card" aria-label="Navigasi pengaturan">
    {#each tabs as t}
      <button
        class="settings-tab"
        class:active={activeTab === t.id}
        on:click={() => (activeTab = t.id)}
        aria-current={activeTab === t.id ? 'page' : undefined}
      >
        <svelte:component this={t.icon} size={16} />
        {t.label}
      </button>
    {/each}
  </nav>

  <div class="settings-content">
    {#if loading}
      <div class="card" style="padding:24px;display:flex;flex-direction:column;gap:14px;">
        {#each Array(5) as _}<div class="skel" style="height:38px;"></div>{/each}
      </div>
    {:else}

      {#if activeTab === 'venue'}
        <div class="settings-section card">
          <div class="section-header"><Building2 size={18} /><div><h3>Informasi Venue</h3><p>Detail tempat wisata atau usaha</p></div></div>
          <div class="section-body">
            <div class="form-grid">
              <div class="field" style="grid-column:1/-1;">
                <label class="label" for="v-name">Nama Tempat</label>
                <input id="v-name" class="input" bind:value={settings.venue.name} placeholder="Nama tempat" />
              </div>
              <div class="field" style="grid-column:1/-1;">
                <label class="label" for="v-addr">Alamat</label>
                <textarea id="v-addr" class="input" rows="2" bind:value={settings.venue.address} placeholder="Alamat lengkap"></textarea>
              </div>
              <div class="field">
                <label class="label" for="v-phone">Telepon</label>
                <input id="v-phone" class="input" type="tel" bind:value={settings.venue.phone} placeholder="021-XXXXXXX" />
              </div>
              <div class="field">
                <label class="label" for="v-email">Email</label>
                <input id="v-email" class="input" type="email" bind:value={settings.venue.email} placeholder="info@venue.id" />
              </div>
              <div class="field">
                <label class="label" for="v-hours">Jam Operasional</label>
                <input id="v-hours" class="input" bind:value={settings.venue.operating_hours} placeholder="09:00 - 21:00" />
              </div>
              <div class="field">
                <label class="label" for="v-footer1">Footer Nota Baris 1</label>
                <input id="v-footer1" class="input" bind:value={settings.venue.footer_line_1} placeholder="Terima kasih!" />
                <span class="field-hint">Baris pertama di bawah struk/nota cetak</span>
              </div>
              <div class="field">
                <label class="label" for="v-footer2">Footer Nota Baris 2</label>
                <input id="v-footer2" class="input" bind:value={settings.venue.footer_line_2} placeholder="Nama Tempat / Slogan" />
                <span class="field-hint">Baris kedua di bawah struk/nota cetak</span>
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === 'ticket'}
        <div class="settings-section card">
          <div class="section-header"><Ticket size={18} /><div><h3>Pengaturan Tiket</h3><p>Konfigurasi sistem tiket</p></div></div>
          <div class="section-body">
            <div class="toggle-list">
              <div class="toggle-row">
                <div><span class="toggle-label">Print Tiket</span><span class="toggle-desc">Cetak tiket otomatis setelah transaksi</span></div>
                <label class="switch" aria-label="Toggle print tiket"><input type="checkbox" bind:checked={settings.ticket.print_enabled} /><span class="switch-track"></span></label>
              </div>
              <div class="toggle-row">
                <div><span class="toggle-label">QR Code Tiket</span><span class="toggle-desc">Generate QR code unik di setiap tiket</span></div>
                <label class="switch" aria-label="Toggle QR tiket"><input type="checkbox" bind:checked={settings.ticket.qr_enabled} /><span class="switch-track"></span></label>
              </div>
            </div>
            <div class="form-grid" style="margin-top:16px;">
              <div class="field">
                <label class="label" for="t-prefix">Prefix Tiket</label>
                <input id="t-prefix" class="input" bind:value={settings.ticket.ticket_prefix} placeholder="TK" maxlength="8" />
              </div>
              <div class="field">
                <label class="label" for="t-expiry">Masa Berlaku (jam, 0=tak terbatas)</label>
                <input id="t-expiry" class="input" type="number" min="0" bind:value={settings.ticket.expiry_hours} />
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === 'printer'}
        <div class="settings-section card">
          <div class="section-header"><Printer size={18} /><div><h3>Pengaturan Printer</h3><p>Konfigurasi Bluetooth thermal printer</p></div></div>
          <div class="section-body">
            <div class="bt-status" class:connected={btConnected}>
              <div style="display:flex;align-items:center;gap:10px;">
                {#if btConnected}<Bluetooth size={20} />{:else}<BluetoothOff size={20} />{/if}
                <div>
                  <span style="font-weight:600;">{btConnected ? 'Printer Terhubung' : 'Printer Tidak Terhubung'}</span>
                  <span style="font-size:0.8125rem;color:var(--text-2);display:block;">{btConnected ? 'Siap mencetak' : 'Klik untuk menghubungkan'}</span>
                </div>
              </div>
              <button class="btn {btConnected ? 'btn-secondary' : 'btn-primary'} btn-sm"
                on:click={() => btConnected ? bluetooth.disconnect() : bluetooth.connect()}>
                {btConnected ? 'Putuskan' : 'Hubungkan'}
              </button>
            </div>
            <div class="toggle-list" style="margin-top:16px;">
              <div class="toggle-row">
                <div><span class="toggle-label">Auto Reconnect</span><span class="toggle-desc">Otomatis hubungkan ulang jika terputus</span></div>
                <label class="switch" aria-label="Toggle auto reconnect"><input type="checkbox" bind:checked={settings.printer.auto_retry} /><span class="switch-track"></span></label>
              </div>
            </div>
            <div class="form-grid" style="margin-top:16px;">
              <div class="field">
                <label class="label" for="p-retry">Maks. Retry</label>
                <input id="p-retry" class="input" type="number" min="1" max="10" bind:value={settings.printer.retry_count} />
              </div>
              <div class="field">
                <label class="label" for="p-paper">Lebar Kertas</label>
                <select id="p-paper" class="input" bind:value={settings.printer.paper_width}>
                  <option value="58mm">58mm</option>
                  <option value="80mm">80mm</option>
                </select>
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === 'shift'}
        <div class="settings-section card">
          <div class="section-header"><Settings size={18} /><div><h3>Pengaturan Kasir</h3><p>Konfigurasi shift dan operasional</p></div></div>
          <div class="section-body">
            <div class="toggle-list">
              <div class="toggle-row">
                <div><span class="toggle-label">Sistem Shift</span><span class="toggle-desc">Aktifkan manajemen shift kasir</span></div>
                <label class="switch" aria-label="Toggle shift"><input type="checkbox" bind:checked={settings.shift.shift_enabled} /><span class="switch-track"></span></label>
              </div>
              <div class="toggle-row">
                <div><span class="toggle-label">Auto Closing</span><span class="toggle-desc">Tutup kasir otomatis di akhir shift</span></div>
                <label class="switch" aria-label="Toggle auto closing"><input type="checkbox" bind:checked={settings.shift.auto_closing} /><span class="switch-track"></span></label>
              </div>
            </div>
            <div class="form-grid" style="margin-top:16px;">
              <div class="field">
                <label class="label" for="s-balance">Saldo Awal Shift (Rp)</label>
                <input id="s-balance" class="input" type="number" min="0" bind:value={settings.shift.opening_balance} />
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === 'system'}
        <div class="settings-section card">
          <div class="section-header"><Database size={18} /><div><h3>Sistem</h3><p>Konfigurasi teknis dan tampilan</p></div></div>
          <div class="section-body">
            <div class="form-grid">
              <div class="field">
                <label class="label" for="s-theme">Tema Tampilan</label>
                <select id="s-theme" class="input" value={$theme} on:change={changeTheme}>
                  <option value="light">Terang</option>
                  <option value="dark">Gelap</option>
                  <option value="system">Ikuti Sistem</option>
                </select>
              </div>
              <div class="field">
                <label class="label" for="s-timeout">Session Timeout (menit)</label>
                <input id="s-timeout" class="input" type="number" min="30" bind:value={settings.system.session_timeout_min} />
              </div>
              <div class="field">
                <label class="label" for="s-cache">Cache TTL (detik)</label>
                <input id="s-cache" class="input" type="number" min="60" bind:value={settings.system.cache_ttl_sec} />
              </div>
            </div>
            <div class="danger-zone">
              <h4>Zona Berbahaya</h4>
              <div style="display:flex;gap:8px;flex-wrap:wrap;">
                <button class="btn btn-secondary btn-sm" style="gap:6px;"><RefreshCw size={13} /> Clear Cache</button>
                <button class="btn btn-secondary btn-sm" style="gap:6px;"><HardDrive size={13} /> Backup Data</button>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <div style="display:flex;justify-content:flex-end;">
        <button class="btn btn-primary" on:click={saveSettings} disabled={saving}>
          {#if saving}<Spinner size={14} color="#fff" />{:else}<Save size={14} />{/if}
          Simpan Perubahan
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-layout { display:grid; grid-template-columns:200px 1fr; gap:20px; align-items:start; max-width:1400px; }
  .settings-nav { padding:6px; display:flex; flex-direction:column; gap:2px; position:sticky; top:calc(var(--topbar-h) + 24px); }
  .settings-tab { display:flex; align-items:center; gap:8px; padding:9px 12px; border-radius:var(--r-md); font-size:0.875rem; font-weight:500; color:var(--text-2); cursor:pointer; border:none; background:none; font-family:var(--font-sans); text-align:left; width:100%; transition:all var(--ease-fast); }
  .settings-tab:hover  { background:var(--bg-muted); color:var(--text-1); }
  .settings-tab.active { background:var(--brand-50); color:var(--brand-600); font-weight:600; }
  [data-theme='dark'] .settings-tab.active { background:rgba(99,102,241,0.14); color:var(--brand-300); }
  .settings-content { display:flex; flex-direction:column; gap:16px; }
  .settings-section { overflow:hidden; }
  .section-header { display:flex; align-items:center; gap:12px; padding:18px 22px; border-bottom:1px solid var(--border); color:var(--text-2); }
  .section-header h3 { font-size:0.9375rem; color:var(--text-1); margin-bottom:2px; }
  .section-header p  { font-size:0.8125rem; color:var(--text-2); }
  .section-body { padding:20px 22px; }
  .form-grid { display:grid; grid-template-columns:1fr 1fr; gap:16px; }
  .toggle-list { display:flex; flex-direction:column; gap:0; }
  .toggle-row { display:flex; align-items:center; justify-content:space-between; padding:14px 0; border-bottom:1px solid var(--border-subtle); gap:16px; }
  .toggle-row:last-child { border-bottom:none; }
  .toggle-label { font-size:0.875rem; font-weight:500; color:var(--text-1); display:block; }
  .toggle-desc  { font-size:0.8125rem; color:var(--text-2); margin-top:2px; }
  .switch { position:relative; display:inline-block; width:40px; height:22px; flex-shrink:0; cursor:pointer; }
  .switch input { opacity:0; width:0; height:0; }
  .switch-track { position:absolute; inset:0; background:var(--gray-300); border-radius:var(--r-full); transition:background var(--ease-base); }
  .switch-track::before { content:''; position:absolute; width:16px; height:16px; left:3px; top:3px; background:#fff; border-radius:var(--r-full); transition:transform var(--ease-base); box-shadow:var(--shadow-xs); }
  .switch input:checked + .switch-track { background:var(--brand-500); }
  .switch input:checked + .switch-track::before { transform:translateX(18px); }
  [data-theme='dark'] .switch-track { background:var(--gray-600); }
  .bt-status { display:flex; align-items:center; justify-content:space-between; padding:14px 16px; border-radius:var(--r-md); background:var(--bg-subtle); border:1px solid var(--border); color:var(--text-2); gap:12px; flex-wrap:wrap; }
  .bt-status.connected { color:var(--emerald); border-color:rgba(16,185,129,0.3); background:rgba(16,185,129,0.06); }
  .danger-zone { margin-top:24px; padding:16px; border-radius:var(--r-md); border:1px solid rgba(244,63,94,0.2); background:rgba(244,63,94,0.04); }
  .danger-zone h4 { font-size:0.875rem; font-weight:600; color:var(--rose); margin-bottom:12px; }
  @media (max-width:768px) { .settings-layout { grid-template-columns:1fr; } .settings-nav { position:static; flex-direction:row; flex-wrap:wrap; } .form-grid { grid-template-columns:1fr; } }
</style>
