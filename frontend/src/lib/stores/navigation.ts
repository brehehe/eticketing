import { writable } from 'svelte/store';

// Global navigation store — updated imperatively on every navigation
function createNavStore() {
  const { subscribe, set } = writable(
    typeof window !== 'undefined' ? window.location.pathname : '/'
  );

  return {
    subscribe,
    set,
    // Call this from afterNavigate
    sync: (path: string) => set(path),
  };
}

export const currentRoute = createNavStore();
