<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { theme } from '$lib/stores/theme';
  import { auth } from '$lib/stores/auth';

  // SvelteKit passes these props to all layouts/pages
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;

  onMount(() => {
    theme.init();
    auth.init();
  });
</script>

<slot />
