import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import {
  SelectionActions,
  type SelectionActionsProps,
} from '@shared/presentation/ui/controls/SelectionActions.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';

interface FeatureHeaderProps extends SelectionActionsProps {
  count?: number;
  label: ReactNode;
  underLabel?: ReactNode;
  pluralLabel?: ReactNode;
  onNew?: () => void;
  newLabel?: ReactNode;
  /** When false, the "New" button is shown disabled with {@link newDeniedReason}. */
  canNew?: boolean;
  newDeniedReason?: ReactNode;
  extraActions?: ReactNode;
}

/**
 * A list screen's title bar: what the list is, how many there are, and the actions
 * that create one. Bulk actions over the selection are accepted here too — but a
 * list rendered with `DataTable` should pass them to its toolbar instead, where they
 * sit next to the rows they act on.
 */
export const FeatureHeader = ({
  count,
  label,
  pluralLabel,
  onNew,
  newLabel,
  canNew = true,
  newDeniedReason,
  extraActions,
  underLabel,
  ...selection
}: FeatureHeaderProps) => {
  const displayLabel = count && count > 1 ? (pluralLabel ?? label) : label;

  return (
    <div className={'flex flex-row items-end justify-between w-full'}>
      <div className={'flex flex-col gap-2'}>
        <div className={'flex flex-row gap-4'}>
          <div className='flex items-baseline gap-2'>
            <h1 className='text-3xl font-bold tracking-tight'>
              {count !== undefined && <span className='text-primary mr-2 '>{count}</span>}
              <span className='text-foreground'>{displayLabel}</span>
            </h1>
            {count !== undefined && (
              <span className='text-sm text-muted-foreground font-medium'>
                <Trans>in total</Trans>
              </span>
            )}
          </div>
        </div>
        {underLabel && <>{underLabel}</>}
      </div>

      <div className={'flex items-center justify-end gap-2'}>
        {/* "Select all" is pointless with nothing to select — `count` is that number here. */}
        <SelectionActions selectableCount={count} {...selection} />
        {extraActions}
        {onNew &&
          (canNew ? (
            <Button onClick={onNew}>{newLabel ?? <Trans>New {label}</Trans>}</Button>
          ) : (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className='inline-flex'>
                  <Button disabled className='pointer-events-none'>
                    {newLabel ?? <Trans>New {label}</Trans>}
                  </Button>
                </span>
              </TooltipTrigger>
              <TooltipContent>
                <p>{newDeniedReason ?? <Trans>You don't have permission to do this.</Trans>}</p>
              </TooltipContent>
            </Tooltip>
          ))}
      </div>
    </div>
  );
};
