import type { ReactNode } from 'react';
import { Check, ListFilter } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { Button } from '@shadcn';
import { cn } from '@shared/presentation/utils';
import { Trans } from '@lingui/react/macro';

export interface DataTableFacetOption {
  value: string;
  label: ReactNode;
  icon?: ReactNode;
}

/**
 * A category filter. It reads its value straight from the row rather than from a
 * column, so it works on tables whose columns are display-only (most of ours).
 */
export interface DataTableFacet<TData> {
  id: string;
  label: ReactNode;
  /** Value the rows are grouped and filtered by. Return undefined to opt a row out. */
  value: (row: TData) => string | undefined;
  /** Labels, order and icons for the values. Omitted → derived from the data. */
  options?: DataTableFacetOption[];
}

/** A facet resolved against the current data: options in display order, with counts. */
export interface FacetView {
  id: string;
  label: ReactNode;
  options: (DataTableFacetOption & { count: number })[];
  selected: string[];
}

interface DataTableFacetFilterProps {
  facet: FacetView;
  onToggle: (value: string) => void;
  onClear: () => void;
}

export const DataTableFacetFilter = ({ facet, onToggle, onClear }: DataTableFacetFilterProps) => {
  const selected = new Set(facet.selected);
  const selectedOptions = facet.options.filter(option => selected.has(option.value));

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant='outline'
          className={cn(
            'max-w-64 rounded-full border font-medium',
            selected.size
              ? 'border-primary/35 bg-primary/[0.08] text-foreground hover:bg-primary/[0.12]'
              : 'border-border/70 text-muted-foreground hover:text-foreground',
          )}
        >
          <ListFilter className='size-3.5 shrink-0 opacity-70' />
          <span className='shrink-0'>{facet.label}</span>
          {selected.size > 0 && (
            <span className='truncate text-primary'>
              {selected.size <= 2 ? (
                selectedOptions.map((option, index) => (
                  <span key={option.value}>
                    {index > 0 && ', '}
                    {option.label}
                  </span>
                ))
              ) : (
                <Trans>{selected.size} selected</Trans>
              )}
            </span>
          )}
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align='start' className='min-w-52'>
        {facet.options.length === 0 && (
          <DropdownMenuItem disabled>
            <Trans>No option</Trans>
          </DropdownMenuItem>
        )}

        {facet.options.map(option => {
          const isSelected = selected.has(option.value);
          return (
            <DropdownMenuItem
              key={option.value}
              // Keep the menu open so several values can be toggled in one go.
              onSelect={event => {
                event.preventDefault();
                onToggle(option.value);
              }}
              className='gap-2'
            >
              <span
                className={cn(
                  'flex size-4 shrink-0 items-center justify-center rounded-[5px] border transition-colors',
                  isSelected
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border',
                )}
              >
                {isSelected && <Check className='size-3' strokeWidth={3} />}
              </span>
              {option.icon}
              <span className='truncate'>{option.label}</span>
              <span className='ml-auto pl-2 text-xs tabular-nums text-muted-foreground'>
                {option.count}
              </span>
            </DropdownMenuItem>
          );
        })}

        {selected.size > 0 && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={onClear}>
              <Trans>Clear filter</Trans>
            </DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
