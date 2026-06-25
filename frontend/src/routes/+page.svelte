<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { goto } from '$app/navigation';
  import { isAuthenticated, auth } from '$lib/stores/auth';
  import { onMount } from 'svelte';

  onMount(() => {
    auth.init();
    // Restore last visited page if authenticated
    if ($isAuthenticated) {
      const last = typeof localStorage !== 'undefined'
        ? localStorage.getItem('tk-last-path')
        : null;
      goto(last && last !== '/' ? last : '/dashboard');
    } else {
      goto('/login');
    }
  });
</script>

<div style="display:flex;align-items:center;justify-content:center;min-height:100dvh;background:var(--bg-base);">
  <div style="width:32px;height:32px;border-radius:50%;border:3px solid var(--brand-500);border-top-color:transparent;animation:spin 0.7s linear infinite;"></div>
</div>

<style>
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
