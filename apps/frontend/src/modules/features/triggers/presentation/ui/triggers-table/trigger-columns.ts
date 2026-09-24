import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
import { triggersMessages } from '../triggers.messages.ts';

export interface TriggerCells {
  name: Snippet<[TriggerEntity]>;
  source: Snippet<[TriggerEntity]>;
  status: Snippet<[TriggerEntity]>;
  enabled: Snippet<[TriggerEntity]>;
  actions: Snippet<[TriggerEntity]>;
}

export const triggerColumns = (cells: TriggerCells): DataTableColumn<TriggerEntity>[] => [
  {
    id: 'name',
    header: t(triggersMessages.name),
    cell: ({ row }) => renderSnippet(cells.name, row.original),
    // Sized, so it does not absorb the free width: the source is what is worth reading.
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
    // Keeps the compact menu reachable.
    size: 100,
    minSize: 80,
  },
];
