import { create } from 'zustand';

export type ToastVariant = 'default' | 'success' | 'error' | 'warning';

export type ToastData = {
  id: string;
  title: string;
  description?: string;
  variant: ToastVariant;
  open: boolean;
};

interface ToastStore {
  toasts: ToastData[];
  addToast: (toast: Omit<ToastData, 'id' | 'open'>) => void;
  dismissToast: (id: string) => void;
  removeToast: (id: string) => void;
}

const MAX_TOASTS = 5;

export const useToastStore = create<ToastStore>(set => ({
  toasts: [],
  addToast: toast =>
    set(state => {
      const id = crypto.randomUUID();
      const next = [...state.toasts, { ...toast, id, open: true }];
      return { toasts: next.slice(-MAX_TOASTS) };
    }),
  dismissToast: id =>
    set(state => ({
      toasts: state.toasts.map(t => (t.id === id ? { ...t, open: false } : t)),
    })),
  removeToast: id => set(state => ({ toasts: state.toasts.filter(t => t.id !== id) })),
}));
