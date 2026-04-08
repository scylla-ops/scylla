import { Toast } from 'radix-ui';
import { useToastStore } from '@shared/presentation/stores/useToastStore.ts';
import { ToastRoot, ToastTitle, ToastDescription, ToastClose, ToastViewport } from './toast.tsx';

export const Toaster = () => {
  const { toasts, dismissToast, removeToast } = useToastStore();

  return (
    <Toast.Provider duration={5000} swipeDirection="right">
      {toasts.map(t => (
        <ToastRoot
          key={t.id}
          variant={t.variant}
          open={t.open}
          onOpenChange={open => {
            if (!open) dismissToast(t.id);
          }}
          onAnimationEnd={e => {
            if (e.animationName === 'toast-slide-out' && !t.open) {
              removeToast(t.id);
            }
          }}
        >
          <ToastTitle>{t.title}</ToastTitle>
          {t.description && <ToastDescription>{t.description}</ToastDescription>}
          <ToastClose />
        </ToastRoot>
      ))}
      <ToastViewport />
    </Toast.Provider>
  );
};
