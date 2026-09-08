import type { ReactNode } from 'react';
import { Search, X } from 'lucide-react';
import { Button, Input } from '@shadcn';
import { cn } from '@shared/presentation/utils';
import { Plural, Trans, useLingui } from '@lingui/react/macro';
import {
  DataTableFacetFilter,
  type FacetView,
} from '@shared/presentation/ui/data-display/DataTableFacetFilter.tsx';
import {
  SelectionActions,
  type SelectionActionsProps,
} from '@shared/presentation/ui/controls/SelectionActions.tsx';

interface DataTableToolbarProps {
  searchable: boolean;
  searchPlaceholder?: string;
  searchQuery: string;
  onSearchChange: (value: string) => void;
  facets: FacetView[];
  onToggleFacetValue: (facetId: string, value: string) => void;
  onClearFacet: (facetId: string) => void;
  onReset: () => void;
  isFiltered: boolean;
  filteredCount: number;
  totalCount: number;
  /** Bulk actions over the selection. They take the bar over while a row is selected. */
  selection?: SelectionActionsProps;
  /** Extra controls rendered at the end of the toolbar (refresh, export…). */
  actions?: ReactNode;
}

export const DataTableToolbar = ({
  searchable,
  searchPlaceholder,
  searchQuery,
  onSearchChange,
  facets,
  onToggleFacetValue,
  onClearFacet,
  onReset,
  isFiltered,
  filteredCount,
  totalCount,
  selection,
  actions,
}: DataTableToolbarProps) => {
  const { t } = useLingui();

  // While rows are selected, the bar is about them: searching or filtering under a
  // pending bulk delete only hides what it is about to act on.
  const selectedCount = selection?.selectedCount ?? 0;

  if (selection && selectedCount > 0) {
    return (
      <div className='flex flex-wrap items-center gap-2 border-b border-border/60 bg-primary/[0.06] px-3 py-2.5'>
        <span className='px-1 text-sm font-medium'>
          <Plural value={selectedCount} one='# selected' other='# selected' />
        </span>
        <div className='ml-auto flex items-center gap-2'>
          <SelectionActions {...selection} />
          {actions}
        </div>
      </div>
    );
  }

  return (
    <div className='flex flex-wrap items-center gap-2 border-b border-border/60 px-3 py-2.5'>
      {searchable && (
        <div
          className={cn(
            'relative flex h-9 min-w-48 flex-1 items-center gap-2 rounded-full border border-transparent bg-muted/60 px-3 transition-colors duration-150 sm:max-w-xs',
            'focus-within:border-primary/35 focus-within:bg-card focus-within:ring-[3px] focus-within:ring-primary/15',
          )}
        >
          <Search className='size-4 shrink-0 text-muted-foreground' />
          <Input
            type='text'
            value={searchQuery}
            onChange={event => onSearchChange(event.target.value)}
            placeholder={searchPlaceholder ?? t`Search…`}
            // The pill wrapper owns the border, background and focus ring.
            className='h-full border-0 bg-transparent px-0 py-0 text-sm shadow-none focus-visible:border-0 focus-visible:ring-0 dark:bg-transparent'
          />
          {searchQuery && (
            <Button
              variant='ghost'
              size='icon'
              onClick={() => onSearchChange('')}
              aria-label={t`Clear search`}
              className='size-6 shrink-0 rounded-full text-muted-foreground hover:text-foreground'
            >
              <X className='size-3.5' />
            </Button>
          )}
        </div>
      )}

      {facets.map(facet => (
        <DataTableFacetFilter
          key={facet.id}
          facet={facet}
          onToggle={value => onToggleFacetValue(facet.id, value)}
          onClear={() => onClearFacet(facet.id)}
        />
      ))}

      <div className='ml-auto flex items-center gap-2'>
        {isFiltered && (
          <>
            <span className='text-xs tabular-nums text-muted-foreground'>
              <Trans>
                {filteredCount} of {totalCount}
              </Trans>
            </span>
            <Button
              variant='ghost'
              onClick={onReset}
              className='rounded-full text-muted-foreground hover:text-foreground'
            >
              <X className='size-3.5' />
              <Trans>Reset</Trans>
            </Button>
          </>
        )}
        {/* Renders "Select all" alone while nothing is selected — the other actions
            need a selection, and a selection takes the whole bar over above. */}
        {selection && <SelectionActions {...selection} />}
        {actions}
      </div>
    </div>
  );
};
