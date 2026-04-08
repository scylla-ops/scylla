import { useToastStore } from '../stores/useToastStore.ts';
import type { ToastVariant } from '../stores/useToastStore.ts';

type ToastOptions = {
  description?: string;
};

function createToast(title: string, variant: ToastVariant, opts?: ToastOptions) {
  useToastStore.getState().addToast({
    title,
    variant,
    description: opts?.description,
  });
}

export const toast = Object.assign(
  (title: string, opts?: ToastOptions) => createToast(title, 'default', opts),
  {
    success: (title: string, opts?: ToastOptions) => createToast(title, 'success', opts),
    error: (title: string, opts?: ToastOptions) => createToast(title, 'error', opts),
    warning: (title: string, opts?: ToastOptions) => createToast(title, 'warning', opts),
  },
);
