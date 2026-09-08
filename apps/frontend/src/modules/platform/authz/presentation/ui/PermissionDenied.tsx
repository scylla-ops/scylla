import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { ShieldOff } from 'lucide-react';

interface PermissionDeniedProps {
  /** What the user tried to reach, e.g. <Trans>manage roles</Trans>. Optional. */
  message?: ReactNode;
}

/**
 * A calm, full-panel replacement for content the current user may not see.
 * Use it (directly or through {@link RequirePermission}) instead of toasting.
 */
export const PermissionDenied = ({ message }: PermissionDeniedProps) => (
  <div className='flex h-full w-full flex-col items-center justify-center gap-3 py-16 text-center text-muted-foreground'>
    <div className='flex size-12 items-center justify-center rounded-xl bg-slate-100'>
      <ShieldOff className='size-6 text-slate-400' />
    </div>
    <p className='text-sm font-medium text-foreground'>
      <Trans>You don't have the permission</Trans>
    </p>
    <p className='max-w-sm text-sm'>
      {message ?? (
        <Trans>Ask an administrator to grant you access if you think you should have it.</Trans>
      )}
    </p>
  </div>
);

export default PermissionDenied;
