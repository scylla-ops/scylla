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
 * Theme tokens only — same semantics as `STATUS_CONFIG` in
 * `@shared/utils/status-config.ts`, which this mirrors for the states it names
 * differently (`success` = `completed`, `idle` = `unknown`):
 * success → `primary`, running → `info`, failure → `destructive`,
 * interrupted → `warning`, inert → `muted-foreground`.
 * Keep the two in step; a status must not be green here and grey there.
 */
const getStateColors = (state: StatusIndicatorProps['state']) => {
  switch (state) {
    case 'success':
      return {
        dot: 'bg-primary',
        ping: 'bg-primary/60',
        container: 'border-primary/30 text-primary',
      };
    case 'failed':
      return {
        dot: 'bg-destructive',
        ping: 'bg-destructive/60',
        container: 'border-destructive/30 text-destructive',
      };
    case 'running':
      return {
        dot: 'bg-info',
        ping: 'bg-info/60',
        container: 'border-info/30 text-info',
      };
    case 'pending':
      return {
        dot: 'bg-muted-foreground',
        ping: 'bg-muted-foreground/50',
        container: 'border-border text-muted-foreground',
      };
    case 'skipped':
      return {
        dot: 'bg-muted-foreground/70',
        ping: 'bg-muted-foreground/40',
        container: 'border-border text-muted-foreground',
      };
    case 'cancelled':
      return {
        dot: 'bg-warning/80',
        ping: 'bg-warning/50',
        container: 'border-warning/30 text-warning',
      };
    case 'orphaned':
      return {
        dot: 'bg-warning',
        ping: 'bg-warning/60',
        container: 'border-warning/30 text-warning',
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
