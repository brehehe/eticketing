<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import Sidebar from './Sidebar.svelte';
  import Topbar from './Topbar.svelte';
  import Toast from '$components/ui/Toast.svelte';

  export let collapsed = false;
  let mobileOpen = false;

  function toggleMobile() {
    mobileOpen = !mobileOpen;
  }

  afterNavigate(() => {
    mobileOpen = false;
  });
</script>

<div class="app-shell">
  <Sidebar bind:collapsed {mobileOpen} />

  {#if mobileOpen}
    <div
      class="mobile-overlay"
      on:click={() => (mobileOpen = false)}
      role="presentation"
      aria-hidden="true"
    ></div>
  {/if}

  <div class="main-area" style="margin-left: {collapsed ? 'var(--sidebar-mini)' : 'var(--sidebar-w)'}">
    <Topbar onMenuToggle={toggleMobile} />
    <main class="page-content" id="main-content" tabindex="-1">
      <slot />
    </main>
  </div>

  <Toast />
</div>

<style>
  .app-shell { display:flex; min-height:100dvh; background:var(--bg-base); }
  .main-area { flex:1; min-width:0; display:flex; flex-direction:column; transition:margin-left var(--ease-base); }
  .page-content { flex:1; padding:24px; overflow-y:auto; outline:none; }
  .mobile-overlay { position:fixed; inset:0; background:rgba(0,0,0,0.4); z-index:49; backdrop-filter:blur(2px); -webkit-backdrop-filter:blur(2px); }
  @media (max-width:768px) {
    .main-area { margin-left:0 !important; }
    .page-content { padding:16px; }
  }
</style>
