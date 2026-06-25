<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import type { Role, Permission } from '$lib/types';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { Shield, Edit2, Users, Check } from 'lucide-svelte';

  let roles: Role[] = [];
  let allPermissions: { module: string; permissions: Permission[] }[] = [];
  let loading = true;
  let modalOpen = false;
  let editTarget: Role | null = null;
  let saving = false;
  let selectedPerms: string[] = [];
  let editName = '';
  let editDesc = '';

  const permissionsData: Permission[] = [
    { id:'dash.view',     module:'Dashboard',  action:'view',   label:'Lihat Dashboard' },
    { id:'tx.create',     module:'Transaksi',  action:'create', label:'Buat Transaksi' },
    { id:'tx.refund',     module:'Transaksi',  action:'refund', label:'Proses Refund' },
    { id:'tx.view',       module:'Transaksi',  action:'view',   label:'Lihat Transaksi' },
    { id:'ticket.scan',   module:'Tiket',      action:'scan',   label:'Scan Tiket' },
    { id:'ticket.print',  module:'Tiket',      action:'print',  label:'Print Tiket' },
    { id:'product.view',  module:'Produk',     action:'view',   label:'Lihat Produk' },
    { id:'product.create',module:'Produk',     action:'create', label:'Tambah Produk' },
    { id:'product.edit',  module:'Produk',     action:'edit',   label:'Edit Produk' },
    { id:'product.delete',module:'Produk',     action:'delete', label:'Hapus Produk' },
    { id:'user.view',     module:'Pengguna',   action:'view',   label:'Lihat Pengguna' },
    { id:'user.create',   module:'Pengguna',   action:'create', label:'Tambah Pengguna' },
    { id:'user.edit',     module:'Pengguna',   action:'edit',   label:'Edit Pengguna' },
    { id:'user.delete',   module:'Pengguna',   action:'delete', label:'Hapus Pengguna' },
    { id:'report.view',   module:'Laporan',    action:'view',   label:'Lihat Laporan' },
    { id:'report.export', module:'Laporan',    action:'export', label:'Export Laporan' },
    { id:'settings.edit', module:'Pengaturan', action:'edit',   label:'Ubah Pengaturan' },
    { id:'investor.view', module:'Investor',   action:'view',   label:'Lihat Data Investor' },
  ];

  const demoRoles: Role[] = [
    {
      id:'1', name:'Admin', slug:'admin', description:'Akses penuh ke seluruh sistem',
      permissions: permissionsData.map(p => p.id),
      user_count: 1,
    },
    {
      id:'2', name:'Kasir', slug:'kasir', description:'Pengelolaan transaksi dan cetak tiket',
      permissions: ['dash.view','tx.create','tx.view','ticket.scan','ticket.print','product.view'],
      user_count: 3,
    },
    {
      id:'3', name:'Officer', slug:'officer', description:'Scan dan validasi tiket di wahana',
      permissions: ['dash.view','ticket.scan','product.view'],
      user_count: 4,
    },
    {
      id:'4', name:'Investor', slug:'investor', description:'Monitoring pendapatan investasi',
      permissions: ['dash.view','report.view','report.export','product.view'],
      user_count: 2,
    },
  ];

  onMount(async () => {
    await new Promise(r => setTimeout(r, 300));
    roles = demoRoles;
    // Group permissions by module
    const modules = [...new Set(permissionsData.map(p => p.module))];
    allPermissions = modules.map(mod => ({
      module: mod,
      permissions: permissionsData.filter(p => p.module === mod),
    }));
    loading = false;
  });

  function openEdit(r: Role) {
    editTarget = r;
    editName = r.name;
    editDesc = r.description;
    selectedPerms = [...r.permissions];
    modalOpen = true;
  }

  function togglePerm(id: string) {
    if (selectedPerms.includes(id)) selectedPerms = selectedPerms.filter(p => p !== id);
    else selectedPerms = [...selectedPerms, id];
  }

  function toggleModule(perms: Permission[]) {
    const ids = perms.map(p => p.id);
    const allSelected = ids.every(id => selectedPerms.includes(id));
    if (allSelected) selectedPerms = selectedPerms.filter(id => !ids.includes(id));
    else selectedPerms = [...new Set([...selectedPerms, ...ids])];
  }

  function moduleAllSelected(perms: Permission[]) {
    return perms.every(p => selectedPerms.includes(p.id));
  }

  async function save() {
    if (!editTarget) return;
    saving = true;
    await new Promise(r => setTimeout(r, 400));
    roles = roles.map(r => r.id === editTarget!.id
      ? { ...r, name:editName, description:editDesc, permissions:selectedPerms }
      : r
    );
    saving = false; modalOpen = false;
  }

  const roleVariant = (slug: string): any => ({ admin:'brand', kasir:'success', officer:'warning', investor:'info' })[slug] ?? 'neutral';
</script>

<svelte:head><title>Role & Akses \u2014 TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Role & Akses</h1>
      <p style="color:var(--text-2);font-size:0.875rem;margin-top:2px;">Kelola hak akses setiap role pengguna</p>
    </div>
  </div>

  {#if loading}
    <div style="display:grid;grid-template-columns:repeat(2,1fr);gap:16px;">
      {#each Array(4) as _}<div class="skel" style="height:160px;"></div>{/each}
    </div>
  {:else}
    <div class="roles-grid">
      {#each roles as role (role.id)}
        <div class="role-card card">
          <div class="role-header">
            <div class="role-icon">
              <Shield size={20} style="color:var(--brand-500);" />
            </div>
            <div style="flex:1;min-width:0;">
              <div style="display:flex;align-items:center;gap:8px;">
                <h3 style="font-size:1rem;">{role.name}</h3>
                <Badge variant={roleVariant(role.slug)}>{role.slug}</Badge>
              </div>
              <p style="font-size:0.8125rem;color:var(--text-2);margin-top:2px;">{role.description}</p>
            </div>
            <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEdit(role)} aria-label="Edit {role.name}">
              <Edit2 size={15} />
            </button>
          </div>

          <div class="role-meta">
            <span style="display:flex;align-items:center;gap:5px;font-size:0.8125rem;color:var(--text-2);">
              <Users size={13} /> {role.user_count} pengguna
            </span>
            <span style="font-size:0.8125rem;color:var(--text-2);">{role.permissions.length} izin aktif</span>
          </div>

          <div class="perm-tags">
            {#each role.permissions.slice(0,6) as pid}
              {@const p = permissionsData.find(x => x.id === pid)}
              {#if p}
                <span class="perm-tag">{p.label}</span>
              {/if}
            {/each}
            {#if role.permissions.length > 6}
              <span class="perm-tag more">+{role.permissions.length - 6} lainnya</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Edit permissions modal -->
<Modal bind:open={modalOpen} title="Edit Izin: {editTarget?.name ?? ''}" size="lg">
  <div class="modal-body">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-bottom:20px;">
      <div class="field">
        <label class="label" for="r-name">Nama Role</label>
        <input id="r-name" class="input" bind:value={editName} />
      </div>
      <div class="field">
        <label class="label" for="r-desc">Deskripsi</label>
        <input id="r-desc" class="input" bind:value={editDesc} />
      </div>
    </div>

    <p class="label" style="margin-bottom:12px;">Izin Akses</p>
    <div style="display:flex;flex-direction:column;gap:12px;">
      {#each allPermissions as group}
        <div class="perm-group">
          <div class="perm-group-header">
            <label style="display:flex;align-items:center;gap:8px;cursor:pointer;">
              <input
                type="checkbox"
                checked={moduleAllSelected(group.permissions)}
                on:change={() => toggleModule(group.permissions)}
                style="accent-color:var(--brand-500);"
              />
              <span style="font-size:0.875rem;font-weight:600;">{group.module}</span>
            </label>
          </div>
          <div class="perm-items">
            {#each group.permissions as perm}
              <label class="perm-item">
                <input
                  type="checkbox"
                  checked={selectedPerms.includes(perm.id)}
                  on:change={() => togglePerm(perm.id)}
                  style="accent-color:var(--brand-500);"
                />
                <span>{perm.label}</span>
              </label>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (modalOpen = false)}>Batal</button>
    <button class="btn btn-primary" on:click={save} disabled={saving}>
      {#if saving}<Spinner size={14} color="#fff" />{/if}
      <Check size={14} /> Simpan
    </button>
  </div>
</Modal>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1000px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }

  .roles-grid { display:grid;grid-template-columns:repeat(2,1fr);gap:16px; }

  .role-card { overflow:hidden; }
  .role-header {
    display:flex;align-items:flex-start;gap:12px;
    padding:18px 18px 14px;border-bottom:1px solid var(--border);
  }
  .role-icon {
    width:40px;height:40px;border-radius:var(--r-md);
    background:var(--brand-50);display:flex;align-items:center;justify-content:center;flex-shrink:0;
  }
  [data-theme='dark'] .role-icon { background:rgba(99,102,241,0.12); }
  .role-meta {
    display:flex;align-items:center;justify-content:space-between;
    padding:10px 18px;border-bottom:1px solid var(--border);
  }
  .perm-tags { display:flex;flex-wrap:wrap;gap:5px;padding:12px 18px; }
  .perm-tag {
    font-size:0.6875rem;font-weight:500;
    padding:2px 7px;border-radius:var(--r-full);
    background:var(--bg-muted);color:var(--text-2);
  }
  .perm-tag.more { color:var(--brand-600);background:var(--brand-50); }
  [data-theme='dark'] .perm-tag.more { background:rgba(99,102,241,0.12);color:var(--brand-300); }

  /* Permission groups */
  .perm-group {
    border:1px solid var(--border);border-radius:var(--r-md);overflow:hidden;
  }
  .perm-group-header {
    padding:10px 14px;background:var(--bg-subtle);
    border-bottom:1px solid var(--border);
  }
  .perm-items {
    display:grid;grid-template-columns:repeat(2,1fr);gap:0;
    padding:4px;
  }
  .perm-item {
    display:flex;align-items:center;gap:8px;
    padding:7px 10px;border-radius:var(--r-sm);
    font-size:0.8125rem;color:var(--text-2);
    cursor:pointer;transition:background var(--ease-fast);
  }
  .perm-item:hover { background:var(--bg-muted);color:var(--text-1); }

  @media (max-width:700px) {
    .roles-grid { grid-template-columns:1fr; }
    .perm-items { grid-template-columns:1fr; }
  }
</style>
