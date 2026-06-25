import { writable } from 'svelte/store';
import type { CartItem } from '$lib/types';

export interface Draft {
  id: string;
  name: string;
  items: CartItem[];
  discount: number;
  discountType: 'nominal' | 'persen';
  voucherCode: string;
  savedAt: string;
}

const STORAGE_KEY = 'tk-drafts';

function load(): Draft[] {
  if (typeof localStorage === 'undefined') return [];
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]'); } catch { return []; }
}

function persist(list: Draft[]) {
  if (typeof localStorage !== 'undefined')
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
}

function createDraftStore() {
  const { subscribe, update, set } = writable<Draft[]>(load());

  return {
    subscribe,
    save(items: CartItem[], opts: { name?: string; discount?: number; discountType?: 'nominal'|'persen'; voucherCode?: string }): string {
      const draft: Draft = {
        id: Date.now().toString(36),
        name: opts.name || `Draft ${new Date().toLocaleTimeString('id-ID', { hour:'2-digit', minute:'2-digit' })}`,
        items: JSON.parse(JSON.stringify(items)), // deep clone
        discount: opts.discount ?? 0,
        discountType: opts.discountType ?? 'nominal',
        voucherCode: opts.voucherCode ?? '',
        savedAt: new Date().toISOString(),
      };
      update(list => { const u = [draft, ...list].slice(0, 10); persist(u); return u; });
      return draft.id;
    },
    remove(id: string) {
      update(list => { const u = list.filter(d => d.id !== id); persist(u); return u; });
    },
    clear() { set([]); persist([]); },
  };
}

export const drafts = createDraftStore();
