import * as React from 'react';
import { Toast } from 'radix-ui';
import { cva, type VariantProps } from 'class-variance-authority';
import { X, CheckCircle2, XCircle, AlertTriangle } from 'lucide-react';
import { cn } from '@shared/presentation/utils';
import type { ToastVariant } from '@shared/presentation/stores/useToastStore.ts';

const toastVariants = cva(
  'group pointer-events-auto relative flex w-full items-start gap-3 overflow-hidden rounded-xl border p-4 shadow-xl backdrop-blur-md transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:animate-toast-slide-out data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)] data-[state=open]:animate-toast-slide-in data-[state=closed]:animate-toast-slide-out',
  {
    variants: {
      variant: {
        default: 'border-border/50 bg-background/80 text-foreground',
        success: 'border-success/40 bg-success/10 text-foreground',
        error: 'border-error/40 bg-error/10 text-foreground',
        warning: 'border-warning/40 bg-warning/10 text-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

const variantIcons: Record<ToastVariant, React.ReactNode> = {
  default: null,
  success: <CheckCircle2 className='h-5 w-5 text-success shrink-0 mt-0.5' />,
  error: <XCircle className='h-5 w-5 text-error shrink-0 mt-0.5' />,
  warning: <AlertTriangle className='h-5 w-5 text-warning shrink-0 mt-0.5' />,
};

type ToastRootProps = React.ComponentPropsWithoutRef<typeof Toast.Root> &
  VariantProps<typeof toastVariants> & {
    variant?: ToastVariant;
  };

const ToastRoot = React.forwardRef<React.ComponentRef<typeof Toast.Root>, ToastRootProps>(
  ({ className, variant = 'default', children, ...props }, ref) => {
    return (
      <Toast.Root ref={ref} className={cn(toastVariants({ variant }), className)} {...props}>
        {variantIcons[variant]}
        <div className='flex-1 grid gap-1'>{children}</div>
      </Toast.Root>
    );
  },
);
ToastRoot.displayName = 'ToastRoot';

const ToastTitle = React.forwardRef<
  React.ComponentRef<typeof Toast.Title>,
  React.ComponentPropsWithoutRef<typeof Toast.Title>
>(({ className, ...props }, ref) => (
  <Toast.Title ref={ref} className={cn('text-sm font-semibold', className)} {...props} />
));
ToastTitle.displayName = 'ToastTitle';

const ToastDescription = React.forwardRef<
  React.ComponentRef<typeof Toast.Description>,
  React.ComponentPropsWithoutRef<typeof Toast.Description>
>(({ className, ...props }, ref) => (
  <Toast.Description
    ref={ref}
    className={cn('text-sm text-muted-foreground', className)}
    {...props}
  />
));
ToastDescription.displayName = 'ToastDescription';

const ToastClose = React.forwardRef<
  React.ComponentRef<typeof Toast.Close>,
  React.ComponentPropsWithoutRef<typeof Toast.Close>
>(({ className, ...props }, ref) => (
  <Toast.Close
    ref={ref}
    className={cn(
      'absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-0 transition-opacity hover:text-foreground focus:opacity-100 focus:outline-none group-hover:opacity-100',
      className,
    )}
    aria-label='Close'
    {...props}
  >
    <X className='h-4 w-4' />
  </Toast.Close>
));
ToastClose.displayName = 'ToastClose';

const ToastViewport = React.forwardRef<
  React.ComponentRef<typeof Toast.Viewport>,
  React.ComponentPropsWithoutRef<typeof Toast.Viewport>
>(({ className, ...props }, ref) => (
  <Toast.Viewport
    ref={ref}
    className={cn(
      'fixed bottom-0 right-0 z-[100] flex max-h-screen w-full flex-col-reverse gap-2 p-4 sm:max-w-[420px]',
      className,
    )}
    {...props}
  />
));
ToastViewport.displayName = 'ToastViewport';

export { ToastRoot, ToastTitle, ToastDescription, ToastClose, ToastViewport };
