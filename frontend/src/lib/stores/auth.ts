import { writable, derived } from 'svelte/store';
import type { Session, User } from '$lib/types';

function createAuthStore() {
  const { subscribe, set, update } = writable<Session | null>(null);

  return {
    subscribe,
    set,
    login: (session: Session) => {
      set(session);
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('tk-token', session.token);
        localStorage.setItem('tk-session', JSON.stringify(session));
      }
    },
    logout: () => {
      set(null);
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('tk-token');
        localStorage.removeItem('tk-session');
      }
    },
    init: () => {
      if (typeof localStorage === 'undefined') return;
      const raw = localStorage.getItem('tk-session');
      if (raw) {
        try {
          const session = JSON.parse(raw) as Session;
          if (new Date(session.expires_at) > new Date()) {
            set(session);
          } else {
            localStorage.removeItem('tk-session');
            localStorage.removeItem('tk-token');
          }
        } catch {
          localStorage.removeItem('tk-session');
        }
      }
    }
  };
}

export const auth = createAuthStore();
export const user = derived(auth, $auth => $auth?.user ?? null);
export const isAuthenticated = derived(auth, $auth => $auth !== null);
export const userRole = derived(auth, $auth => $auth?.user?.role ?? null);

export function hasPermission(role: User['role'] | null, ...allowed: User['role'][]): boolean {
  if (!role) return false;
  return allowed.includes(role);
}
