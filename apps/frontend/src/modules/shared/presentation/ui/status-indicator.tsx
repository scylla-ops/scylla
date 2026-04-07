import React from 'react';
import { cn } from '@shared/presentation/utils';

interface StatusIndicatorProps {
  state: 'success' | 'failure' | 'running' | 'idle';
  label?: string;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  labelClassName?: string;
}

const getStateColors = (state: StatusIndicatorProps['state']) => {
  switch (state) {
    case 'success':
      return {
        dot: 'bg-green-500',
        ping: 'bg-green-300',
        container: 'border-green-200 text-green-800 dark:border-green-800 dark:text-green-300',
        gradient: 'from-green-400 to-green-500',
      };
    case 'failure':
      return {
        dot: 'bg-red-500',
        ping: 'bg-red-300',
        container: 'border-red-200 text-red-800 dark:border-red-800 dark:text-red-300',
        gradient: 'from-red-400 to-red-500',
      };
    case 'running':
      return {
        dot: 'bg-blue-500',
        ping: 'bg-blue-300',
        container: 'border-blue-200 text-blue-800 dark:border-blue-800 dark:text-blue-300',
        gradient: 'from-blue-400 to-blue-500',
      };
    case 'idle':
    default:
      return {
        dot: 'bg-slate-700',
        ping: 'bg-slate-400',
        container: 'border-slate-200 text-slate-700 dark:border-slate-700 dark:text-slate-300',
        gradient: 'from-slate-400 to-slate-500',
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
}) => {
  const shouldAnimate = state === 'success' || state === 'running' || state === 'failure';
  const colors = getStateColors(state);
  const sizeClasses = getSizeClasses(size);

  return (
    <div className='relative inline-flex rounded-full p-[2px] overflow-hidden bg-gradient-to-r from-transparent via-transparent to-transparent'>
      {/* Gradient TODO */
      /*{state === 'running' && (
        <span
          className={cn(
            'absolute inset-0 rounded-full animate-spin pointer-events-none opacity-75',
            `bg-gradient-to-r ${colors.gradient}`,
          )}
        />
      )} */}

      <div
        className={cn(
          'relative inline-flex items-center gap-2 rounded-full bg-white dark:bg-slate-900 transition-all duration-300',
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
