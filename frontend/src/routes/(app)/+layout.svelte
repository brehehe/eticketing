<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { isAuthenticated, auth, userRole } from '$lib/stores/auth';
  import { toast } from '$lib/stores/toast';
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';

  // SvelteKit passes these props to all layouts/pages
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;

  const routePermissions: Record<string, string[]> = {
    '/kasir': ['admin', 'kasir'],
    '/tiket': ['admin', 'kasir', 'officer'],
    '/produk': ['admin'],
    '/kategori': ['admin'],
    '/metode-bayar': ['admin'],
    '/users': ['admin'],
    '/roles': ['admin'],
    '/laporan/produk': ['admin', 'investor'],
    '/laporan/investor': ['admin', 'investor'],
    '/laporan/metode-bayar': ['admin'],
    '/laporan': ['admin', 'investor'],
    '/pengaturan': ['admin'],
    '/printer': ['admin', 'kasir'],
    '/dashboard': ['admin', 'kasir', 'officer', 'investor']
  };

  $: {
    if ($isAuthenticated && $userRole) {
      const path = $page.url.pathname;
      const matchedRoute = Object.keys(routePermissions).find(route => {
        if (route === '/dashboard') return path === '/dashboard';
        if (route === '/laporan') return path === '/laporan';
        return path === route || path.startsWith(route + '/');
      });

      if (matchedRoute) {
        const allowedRoles = routePermissions[matchedRoute];
        if (!allowedRoles.includes($userRole)) {
          goto('/dashboard');
          toast.error('Akses ditolak', 'Anda tidak memiliki hak akses untuk halaman tersebut.');
        }
      }
    }
  }

  // Remember last visited page to restore after refresh
  $: if (typeof localStorage !== 'undefined' && $page.url.pathname !== '/login') {
    localStorage.setItem('tk-last-path', $page.url.pathname);
  }

  onMount(() => {
    // Init auth from localStorage
    auth.init();

    if (!$isAuthenticated) {
      // Save current path and redirect to login
      goto('/login');
    }
  });
</script>

{#if $isAuthenticated}
  <AppShell>
    <slot />
  </AppShell>
{:else}
  <!-- Show nothing while checking auth — avoids flash -->
  <div style="min-height:100dvh;background:var(--bg-base);"></div>
{/if}
