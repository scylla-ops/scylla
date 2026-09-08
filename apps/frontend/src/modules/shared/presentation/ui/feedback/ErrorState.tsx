import { Trans } from '@lingui/react/macro';
import type { ReactNode } from 'react';

type ErrorStateProps = {
  title?: ReactNode;
  message?: ReactNode;
};

export const ErrorState = ({ title, message }: ErrorStateProps) => (
  <div className='flex items-center justify-center h-full'>
    <div className='text-center space-y-2'>
      <p className='text-destructive text-lg font-semibold'>{title ?? <Trans>Error</Trans>}</p>
      <p className='text-muted-foreground text-sm'>{message ?? <Trans>An error occurred</Trans>}</p>
    </div>
  </div>
);
