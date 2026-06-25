import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  function add(toast: Omit<Toast, 'id'>): string {
    const id = Math.random().toString(36).slice(2);
    const duration = toast.duration ?? 4000;
    update(list => [...list, { ...toast, id }]);
    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }
    return id;
  }

  function remove(id: string) {
    update(list => list.filter(t => t.id !== id));
  }

  return {
    subscribe,
    add,
    remove,
    success: (title: string, message?: string) => add({ type: 'success', title, message }),
    error:   (title: string, message?: string) => add({ type: 'error',   title, message, duration: 6000 }),
    warning: (title: string, message?: string) => add({ type: 'warning', title, message }),
    info:    (title: string, message?: string) => add({ type: 'info',    title, message }),
  };
}

export const toast = createToastStore();
