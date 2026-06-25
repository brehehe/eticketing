import { writable } from 'svelte/store';
import type { Theme } from '$lib/types';

function createThemeStore() {
  const stored = (typeof localStorage !== 'undefined'
    ? localStorage.getItem('tk-theme')
    : null) as Theme | null;

  const { subscribe, set, update } = writable<Theme>(stored ?? 'system');

  function apply(theme: Theme) {
    if (typeof document === 'undefined') return;
    const root = document.documentElement;
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      root.setAttribute('data-theme', theme);
    }
  }

  return {
    subscribe,
    set: (t: Theme) => {
      set(t);
      if (typeof localStorage !== 'undefined') localStorage.setItem('tk-theme', t);
      apply(t);
    },
    toggle: () => {
      update(t => {
        const next = t === 'light' ? 'dark' : 'light';
        if (typeof localStorage !== 'undefined') localStorage.setItem('tk-theme', next);
        apply(next);
        return next;
      });
    },
    init: () => {
      const t = (typeof localStorage !== 'undefined'
        ? localStorage.getItem('tk-theme') as Theme
        : null) ?? 'system';
      set(t);
      apply(t);

      // Watch system preference
      if (typeof window !== 'undefined') {
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
          if (t === 'system') apply('system');
        });
      }
    }
  };
}

export const theme = createThemeStore();
