import React from 'react';
import { cn } from '@shared/presentation/utils';

export type StatusState =
  | 'success'
  | 'failed'
  | 'running'
  | 'pending'
  | 'idle'
  | 'skipped'
  | 'cancelled'
  | 'orphaned';

interface StatusIndicatorProps {
  state: StatusState;
  label?: string;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  labelClassName?: string;
  animateAllStates?: boolean;
}

/**
 * Uses semantic status tokens from the theme:
 * success → `status-passed`, running → `status-running`, failure → `status-failed`,
 * pending → `status-queued`, skipped → `status-skipped`,
 * cancelled / orphaned → `status-canceled`, idle → `muted-foreground`.
 */
const getStateColors = (state: StatusIndicatorProps['state']) => {
  switch (state) {
    case 'success':
      return {
        dot: 'bg-status-passed',
        ping: 'bg-status-passed/60',
        container: 'border-status-passed/30 text-status-passed',
      };
    case 'failed':
      return {
        dot: 'bg-status-failed',
        ping: 'bg-status-failed/60',
        container: 'border-status-failed/30 text-status-failed',
      };
    case 'running':
      return {
        dot: 'bg-status-running',
        ping: 'bg-status-running/60',
        container: 'border-status-running/30 text-status-running',
      };
    case 'pending':
      return {
        dot: 'bg-status-queued',
        ping: 'bg-status-queued/50',
        container: 'border-status-queued/30 text-status-queued',
      };
    case 'skipped':
      return {
        dot: 'bg-status-skipped',
        ping: 'bg-status-skipped/50',
        container: 'border-status-skipped/30 text-status-skipped',
      };
    case 'cancelled':
    case 'orphaned':
      return {
        dot: 'bg-status-canceled',
        ping: 'bg-status-canceled/50',
        container: 'border-status-canceled/30 text-status-canceled',
      };
    case 'idle':
    default:
      return {
        dot: 'bg-muted-foreground/60',
        ping: 'bg-muted-foreground/30',
        container: 'border-border text-muted-foreground',
      };
  }
};

const getSizeClasses = (size: StatusIndicatorProps['size']) => {
  switch (size) {
    case 'sm':
      return {
        dot: 'h-2 w-2',
        ping: 'h-2 w-2',
        container: 'px-2 py-1 text-xs',
      };
    case 'lg':
      return {
        dot: 'h-4 w-4',
        ping: 'h-4 w-4',
        container: 'px-4 py-2 text-sm',
      };
    case 'md':
    default:
      return {
        dot: 'h-3 w-3',
        ping: 'h-3 w-3',
        container: 'px-3 py-1.5 text-sm',
      };
  }
};

const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  state = 'idle',
  label,
  className,
  size = 'md',
  labelClassName,
  animateAllStates = false,
}) => {
  const shouldAnimate = state === 'running' || state === 'pending' || animateAllStates;
  const colors = getStateColors(state);
  const sizeClasses = getSizeClasses(size);

  return (
    <div className='relative inline-flex rounded-full overflow-hidden'>
      <div
        className={cn(
          'relative inline-flex items-center gap-2 rounded-full bg-card transition-all duration-300',
          sizeClasses.container,
          colors.container,
          className,
        )}
      >
        <div className='relative flex items-center'>
          {shouldAnimate && (
            <span
              className={cn(
                'absolute inline-flex rounded-full opacity-75 animate-ping',
                sizeClasses.ping,
                colors.ping,
              )}
            />
          )}
          <span className={cn('relative inline-flex rounded-full', sizeClasses.dot, colors.dot)} />
        </div>

        {label && <p className={cn('font-medium', labelClassName)}>{label}</p>}
      </div>
    </div>
  );
};

export default StatusIndicator;
