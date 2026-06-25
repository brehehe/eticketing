<script lang="ts">
  export let data: any = {};
  export let params: any = undefined;
  import { onMount } from 'svelte';
  import { relativeTime } from '$lib/utils/format';
  import { usersApi } from '$lib/api/resources';
  import { toast } from '$lib/stores/toast';
  import type { User } from '$lib/types';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import { Plus, Search, Edit2, Trash2 } from 'lucide-svelte';

  let users: User[] = [];
  let loading = true;
  let searchQuery = '';
  let filterRole = 'all';
  let modalOpen = false;
  let deleteOpen = false;
  let editTarget: User | null = null;
  let deleteTarget: User | null = null;
  let saving = false;
  let total = 0;

  let form = { name:'', username:'', email:'', password:'', role:'kasir' as User['role'], status:'active' as 'active'|'inactive' };

  const roles = [
    { id:'admin',    label:'Admin'    },
    { id:'kasir',    label:'Kasir'    },
    { id:'officer',  label:'Officer'  },
    { id:'investor', label:'Investor' },
  ];

  async function loadUsers() {
    loading = true;
    try {
      const params: Record<string,string> = {};
      if (searchQuery) params.search = searchQuery;
      if (filterRole !== 'all') params.role = filterRole;
      const res = await usersApi.list(params);
      const d = (res as any)?.data;
      users = Array.isArray(d) ? d : [];
      total = (res as any)?.total ?? users.length;
    } catch (e: any) {
      toast.error('Gagal memuat pengguna', e.message);
    } finally {
      loading = false;
    }
  }

  onMount(loadUsers);

  let searchTimer: ReturnType<typeof setTimeout>;
  function onSearch() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(loadUsers, 400);
  }

  function openNew() {
    editTarget = null;
    form = { name:'', username:'', email:'', password:'', role:'kasir', status:'active' };
    modalOpen = true;
  }
  function openEdit(u: User) {
    editTarget = u;
    form = { name:u.name, username:u.username, email:u.email, password:'', role:u.role, status:u.status };
    modalOpen = true;
  }

  async function save() {
    if (!form.name || !form.username) return;
    saving = true;
    try {
      if (editTarget) {
        await usersApi.update(editTarget.id, form);
        toast.success('Pengguna diperbarui');
      } else {
        await usersApi.create(form);
        toast.success('Pengguna ditambahkan');
      }
      modalOpen = false;
      await loadUsers();
    } catch (e: any) {
      toast.error('Gagal menyimpan', e.message);
    } finally { saving = false; }
  }

  async function deleteUser() {
    if (!deleteTarget) return;
    saving = true;
    try {
      await usersApi.delete(deleteTarget.id);
      toast.success('Pengguna dihapus');
      deleteOpen = false;
      await loadUsers();
    } catch (e: any) {
      toast.error('Gagal menghapus', e.message);
    } finally { saving = false; }
  }

  const roleVariant = (r: string): any => ({ admin:'brand', kasir:'success', officer:'warning', investor:'info' })[r] ?? 'neutral';
  const roleName    = (r: string) => roles.find(x => x.id === r)?.label ?? r;
</script>

<svelte:head><title>Pengguna — TiketKu</title></svelte:head>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Pengguna</h1>
      <p style="color:var(--text-2);margin-top:2px;font-size:0.875rem;">{total} pengguna terdaftar</p>
    </div>
    <button class="btn btn-primary" on:click={openNew}><Plus size={15} /> Tambah Pengguna</button>
  </div>

  <div class="filters">
    <div class="search-wrap">
      <Search size={15} style="color:var(--text-3);" />
      <input class="search-input" type="search" bind:value={searchQuery} on:input={onSearch} placeholder="Cari nama / username / email..." />
    </div>
    <select class="input input-sm" style="width:140px;" bind:value={filterRole} on:change={loadUsers}>
      <option value="all">Semua Role</option>
      {#each roles as r}<option value={r.id}>{r.label}</option>{/each}
    </select>
  </div>

  {#if loading}
    <div style="display:flex;flex-direction:column;gap:10px;">{#each Array(5) as _}<div class="skel" style="height:60px;"></div>{/each}</div>
  {:else if users.length === 0}
    <div class="empty-state"><p>Tidak ada pengguna ditemukan</p></div>
  {:else}
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr><th>Pengguna</th><th>Username</th><th>Role</th><th>Status</th><th>Login Terakhir</th><th>Device</th><th></th></tr>
        </thead>
        <tbody>
          {#each users as u (u.id)}
            <tr>
              <td>
                <div style="display:flex;align-items:center;gap:10px;">
                  <Avatar name={u.name} size="sm" />
                  <div>
                    <span style="font-weight:600;">{u.name}</span>
                    <span style="font-size:0.75rem;color:var(--text-3);display:block;">{u.email}</span>
                  </div>
                </div>
              </td>
              <td><span class="mono" style="font-size:0.8125rem;">{u.username}</span></td>
              <td><Badge variant={roleVariant(u.role)} dot>{roleName(u.role)}</Badge></td>
              <td><Badge variant={u.status === 'active' ? 'success' : 'neutral'} dot>{u.status === 'active' ? 'Aktif' : 'Nonaktif'}</Badge></td>
              <td style="color:var(--text-2);font-size:0.8125rem;">{u.last_login ? relativeTime(u.last_login) : '—'}</td>
              <td style="color:var(--text-2);font-size:0.8125rem;">{u.device_count} device</td>
              <td>
                <div style="display:flex;gap:4px;">
                  <button class="btn btn-ghost btn-icon btn-sm" on:click={() => openEdit(u)} aria-label="Edit"><Edit2 size={14} /></button>
                  <button class="btn btn-ghost btn-icon btn-sm" style="color:var(--rose);" on:click={() => { deleteTarget=u; deleteOpen=true; }} aria-label="Hapus"><Trash2 size={14} /></button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<Modal bind:open={modalOpen} title={editTarget ? 'Edit Pengguna' : 'Tambah Pengguna'} size="md">
  <div class="modal-body">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
      <div class="field" style="grid-column:1/-1;">
        <label class="label" for="u-name">Nama Lengkap</label>
        <input id="u-name" class="input" bind:value={form.name} placeholder="Nama pengguna" />
      </div>
      <div class="field">
        <label class="label" for="u-username">Username</label>
        <input id="u-username" class="input" bind:value={form.username} placeholder="username" autocapitalize="none" />
      </div>
      <div class="field">
        <label class="label" for="u-email">Email</label>
        <input id="u-email" class="input" type="email" bind:value={form.email} placeholder="email@domain.com" />
      </div>
      <div class="field">
        <label class="label" for="u-pass">{editTarget ? 'Password Baru (kosongkan = tidak berubah)' : 'Password'}</label>
        <input id="u-pass" class="input" type="password" bind:value={form.password} autocomplete="new-password" />
      </div>
      <div class="field">
        <label class="label" for="u-role">Role</label>
        <Select
          id="u-role"
          bind:value={form.role}
          options={roles.map(r => ({ value: r.id, label: r.label }))}
        />
      </div>
      <div class="field">
        <label class="label" for="u-status">Status</label>
        <Select
          id="u-status"
          bind:value={form.status}
          options={[
            { value: 'active',   label: 'Aktif'    },
            { value: 'inactive', label: 'Nonaktif' },
          ]}
        />
      </div>
    </div>
  </div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (modalOpen=false)}>Batal</button>
    <button class="btn btn-primary" on:click={save} disabled={saving || !form.name || !form.username}>
      {#if saving}<Spinner size={14} color="#fff" />{/if} {editTarget ? 'Simpan' : 'Tambah'}
    </button>
  </div>
</Modal>

<Modal bind:open={deleteOpen} title="Hapus Pengguna" size="sm">
  <div class="modal-body"><p>Yakin ingin menghapus <strong>{deleteTarget?.name}</strong>?</p></div>
  <div class="modal-footer">
    <button class="btn btn-secondary" on:click={() => (deleteOpen=false)}>Batal</button>
    <button class="btn btn-danger" on:click={deleteUser} disabled={saving}>{#if saving}<Spinner size={14} color="#fff" />{/if} Hapus</button>
  </div>
</Modal>

<style>
  .page { display:flex;flex-direction:column;gap:20px;max-width:1100px; }
  .page-header { display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px; }
  .page-header h1 { font-size:1.5rem;font-weight:700;letter-spacing:-0.025em; }
  .filters { display:flex;gap:10px;flex-wrap:wrap; }
  .search-wrap { display:flex;align-items:center;gap:8px;padding:0 12px;height:36px;background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-md);flex:1;min-width:200px; }
  .search-input { flex:1;border:none;outline:none;background:transparent;font-family:var(--font-sans);font-size:0.875rem;color:var(--text-1); }
  .search-input::placeholder { color:var(--text-3); }
  .empty-state { padding:48px;text-align:center;color:var(--text-3);background:var(--bg-surface);border:1px solid var(--border);border-radius:var(--r-lg); }
</style>
