import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
import { triggersMessages } from '../triggers.messages.ts';

/** One snippet per column, each taking the row it renders. */
export interface TriggerCells {
  name: Snippet<[TriggerEntity]>;
  source: Snippet<[TriggerEntity]>;
  status: Snippet<[TriggerEntity]>;
  enabled: Snippet<[TriggerEntity]>;
  actions: Snippet<[TriggerEntity]>;
}

/**
 * The triggers table, as a column definition.
 *
 * `size` / `minSize` survived the port untouched — they are the same numbers the
 * React version declared, which is the dividend of TanStack Table being
 * framework-agnostic data. What changed is that a `cell` returns a Svelte
 * snippet instead of a React element, so the markup stays in `TriggersTable`
 * and this file stays a `.ts`.
 */
export const triggerColumns = (cells: TriggerCells): DataTableColumn<TriggerEntity>[] => [
  {
    id: 'name',
    header: t(triggersMessages.name),
    cell: ({ row }) => renderSnippet(cells.name, row.original),
    // Sized like the other columns, so DataTable shares width proportionally
    // instead of letting this be the one flexible column that absorbs whatever
    // a wide screen leaves over. Kept close to its floor: the source (the
    // webhook URL / cron expression) is what's worth reading in full, not the
    // name.
    size: 220,
    minSize: 220,
  },
  {
    id: 'source',
    header: t(triggersMessages.source),
    cell: ({ row }) => renderSnippet(cells.source, row.original),
    size: 380,
    minSize: 200,
  },
  {
    id: 'status',
    header: t(triggersMessages.status),
    cell: ({ row }) => renderSnippet(cells.status, row.original),
    size: 140,
    minSize: 120,
  },
  {
    id: 'enabled',
    header: t(triggersMessages.enabledColumn),
    cell: ({ row }) => renderSnippet(cells.enabled, row.original),
    size: 70,
    minSize: 60,
  },
  {
    id: 'actions',
    header: t(triggersMessages.actions),
    cell: ({ row }) => renderSnippet(cells.actions, row.original),
    // Floored so the compact dropdown stays reachable instead of collapsing away.
    size: 100,
    minSize: 80,
  },
];
