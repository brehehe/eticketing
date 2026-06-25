<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth, userRole } from '$lib/stores/auth';
  import { theme } from '$lib/stores/theme';
  import { page } from '$app/stores';
  import Avatar from '$components/ui/Avatar.svelte';
  import {
    LayoutDashboard, ShoppingCart, Ticket, Package, Users, BarChart3,
    Settings, LogOut, ChevronLeft, ChevronRight, Scan,
    UserCog, TrendingUp, Printer, Moon, Sun, CreditCard, Tag, DollarSign
  } from 'lucide-svelte';

  export let collapsed  = false;
  export let mobileOpen = false;

  type NavItem = { href: string; label: string; icon: any; roles: string[] };
  type Divider = { divider: true; label: string; roles: string[] };

  const nav: (NavItem | Divider)[] = [
    { href:'/dashboard',      label:'Dashboard',    icon:LayoutDashboard, roles:['admin','kasir','officer','investor'] },
    { href:'/kasir',          label:'Kasir / POS',  icon:ShoppingCart,    roles:['admin','kasir'] },
    { href:'/tiket',          label:'Scan Tiket',   icon:Scan,            roles:['admin','kasir','officer'] },
    { divider:true, label:'Master Data', roles:['admin'] },
    { href:'/produk',         label:'Produk',       icon:Package,         roles:['admin'] },
    { href:'/kategori',        label:'Kategori',     icon:Tag,             roles:['admin'] },
    { href:'/metode-bayar',   label:'Metode Bayar', icon:CreditCard,      roles:['admin'] },
    { href:'/users',          label:'Pengguna',     icon:Users,           roles:['admin'] },
    { href:'/roles',          label:'Role & Akses', icon:UserCog,         roles:['admin'] },
    { divider:true, label:'Laporan', roles:['admin','investor'] },
    { href:'/laporan',               label:'Penjualan Harian', icon:BarChart3,    roles:['admin','investor'] },
    { href:'/laporan/produk',        label:'Per Produk',       icon:TrendingUp,   roles:['admin','investor'] },
    { href:'/laporan/metode-bayar',  label:'Metode Bayar',     icon:CreditCard,   roles:['admin'] },
    { href:'/laporan/investor',      label:'Investor',         icon:DollarSign,   roles:['admin','investor'] },
    { divider:true, label:'Sistem', roles:['admin'] },
    { href:'/pengaturan',     label:'Pengaturan',   icon:Settings,        roles:['admin'] },
    { href:'/printer',        label:'Printer',      icon:Printer,         roles:['admin','kasir'] },
  ];

  $: role   = $userRole;
  $: isDark = $theme === 'dark';

  // Use the SvelteKit page store as single source of truth
  $: activePath = $page.url.pathname;

  function isActive(href: string, path: string): boolean {
    if (href === '/dashboard') return path === '/dashboard';
    if (href === '/laporan')   return path === '/laporan';
    return path === href || path.startsWith(href + '/');
  }

  function canSee(roles: string[]): boolean {
    return !!role && roles.includes(role);
  }

  async function logout() {
    auth.logout();
    await goto('/login');
  }
</script>

<aside
  class="sidebar"
  class:collapsed
  class:mobile-open={mobileOpen}
  aria-label="Navigasi utama"
>
  <div class="sidebar-logo">
    <div class="logo-mark">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <rect width="24" height="24" rx="7" fill="var(--brand-500)"/>
        <path d="M7 12h10M12 7l5 5-5 5" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
    {#if !collapsed}
      <span class="logo-text">TiketKu</span>
    {/if}
    <button
      class="collapse-btn btn btn-ghost btn-icon btn-sm"
      on:click={() => (collapsed = !collapsed)}
      aria-label={collapsed ? 'Perluas sidebar' : 'Perkecil sidebar'}
    >
      {#if collapsed}<ChevronRight size={14} />{:else}<ChevronLeft size={14} />{/if}
    </button>
  </div>

  <nav class="sidebar-nav" aria-label="Menu navigasi">
    {#each nav as item (('href' in item) ? item.href : item.label)}
      {#if 'divider' in item}
        {#if canSee(item.roles)}
          {#if !collapsed}
            <div class="nav-section">{item.label}</div>
          {:else}
            <div class="nav-divider"></div>
          {/if}
        {/if}
      {:else if canSee(item.roles)}
        <a
          href={item.href}
          class="nav-item"
          class:active={isActive(item.href, activePath)}
          title={collapsed ? item.label : undefined}
          aria-current={isActive(item.href, activePath) ? 'page' : undefined}
        >
          <span class="nav-icon">
            <svelte:component this={item.icon} size={18} strokeWidth={1.75} />
          </span>
          {#if !collapsed}
            <span class="nav-label">{item.label}</span>
          {/if}
          {#if isActive(item.href, activePath) && !collapsed}
            <span class="nav-active-dot" aria-hidden="true"></span>
          {/if}
        </a>
      {/if}
    {/each}
  </nav>

  <div class="sidebar-footer">
    <button
      class="nav-item"
      on:click={theme.toggle}
      title="Toggle tema"
      aria-label="Toggle tema"
    >
      <span class="nav-icon">
        {#if isDark}<Sun size={18} strokeWidth={1.75} />{:else}<Moon size={18} strokeWidth={1.75} />{/if}
      </span>
      {#if !collapsed}
        <span class="nav-label">{isDark ? 'Tema Terang' : 'Tema Gelap'}</span>
      {/if}
    </button>

    {#if $auth}
      <div class="sidebar-user" class:collapsed>
        <Avatar name={$auth.user.name} size="sm" />
        {#if !collapsed}
          <div class="user-info">
            <span class="user-name">{$auth.user.name}</span>
            <span class="user-role">{$auth.user.role}</span>
          </div>
          <button class="btn btn-ghost btn-icon btn-sm logout-btn" on:click={logout} aria-label="Keluar">
            <LogOut size={15} />
          </button>
        {:else}
          <button class="btn btn-ghost btn-icon btn-sm logout-btn" on:click={logout} aria-label="Keluar">
            <LogOut size={15} />
          </button>
        {/if}
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar { width:var(--sidebar-w); height:100dvh; position:fixed; left:0; top:0; bottom:0; background:var(--bg-surface); border-right:1px solid var(--border); display:flex; flex-direction:column; z-index:50; transition:width var(--ease-base); overflow:hidden; }
  .sidebar.collapsed { width:var(--sidebar-mini); }
  .sidebar-logo { height:var(--topbar-h); display:flex; align-items:center; gap:10px; padding:0 14px; border-bottom:1px solid var(--border); flex-shrink:0; }
  .logo-mark { flex-shrink:0; display:flex; }
  .logo-text { font-size:1rem; font-weight:700; color:var(--text-1); letter-spacing:-0.02em; white-space:nowrap; flex:1; }
  .collapse-btn { margin-left:auto; flex-shrink:0; color:var(--text-3); }
  .collapse-btn:hover { color:var(--text-1); }
  .sidebar-nav { flex:1; overflow-y:auto; padding:8px; display:flex; flex-direction:column; gap:1px; }
  .nav-section { font-size:0.6875rem; font-weight:600; letter-spacing:0.07em; text-transform:uppercase; color:var(--text-3); padding:14px 8px 5px; white-space:nowrap; }
  .nav-divider { height:1px; background:var(--border); margin:8px 4px; }
  .nav-item { display:flex; align-items:center; gap:10px; padding:0 8px; height:38px; border-radius:var(--r-md); font-size:0.875rem; font-weight:500; color:var(--text-2); cursor:pointer; transition:background var(--ease-fast), color var(--ease-fast); white-space:nowrap; text-decoration:none; position:relative; width:100%; text-align:left; border:none; background:none; font-family:var(--font-sans); }
  .nav-item:hover { background:var(--bg-muted); color:var(--text-1); }
  .nav-item.active { background:var(--brand-50); color:var(--brand-600); font-weight:600; }
  [data-theme='dark'] .nav-item.active { background:rgba(99,102,241,0.14); color:var(--brand-300); }
  .nav-icon { flex-shrink:0; display:flex; align-items:center; }
  .nav-label { flex:1; overflow:hidden; text-overflow:ellipsis; }
  .nav-active-dot { width:6px; height:6px; border-radius:var(--r-full); background:var(--brand-500); flex-shrink:0; }
  .sidebar-footer { padding:8px; border-top:1px solid var(--border); display:flex; flex-direction:column; gap:2px; }
  .sidebar-user { display:flex; align-items:center; gap:10px; padding:8px; border-radius:var(--r-md); background:var(--bg-subtle); min-width:0; }
  .sidebar-user.collapsed { justify-content:center; }
  .user-info { flex:1; min-width:0; display:flex; flex-direction:column; gap:1px; }
  .user-name { font-size:0.8125rem; font-weight:600; color:var(--text-1); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .user-role { font-size:0.6875rem; color:var(--text-3); text-transform:capitalize; }
  .logout-btn { flex-shrink:0; color:var(--text-3); }
  .logout-btn:hover { color:var(--rose); }
  @media (max-width:768px) { .sidebar { transform:translateX(-100%); transition:transform var(--ease-base), width var(--ease-base); } .sidebar.mobile-open { transform:translateX(0); } }
</style>
