import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { PipelineMetadata } from '../../../../domain/structs/pipeline.struct.ts';
import { pipelineMessages } from '../../../pipeline.messages.ts';

export interface PipelineCells {
  status: Snippet<[PipelineMetadata]>;
  history: Snippet<[PipelineMetadata]>;
  lastRun: Snippet<[PipelineMetadata]>;
  actions: Snippet<[PipelineMetadata]>;
}

export const pipelineColumns = (cells: PipelineCells): DataTableColumn<PipelineMetadata>[] => [
  {
    id: 'status',
    header: t(pipelineMessages.status),
    cell: ({ row }) => renderSnippet(cells.status, row.original),
    size: 280,
    minSize: 250,
  },
  {
    id: 'history',
    header: t(pipelineMessages.history),
    cell: ({ row }) => renderSnippet(cells.history, row.original),
    // No size: takes the space left, down to 180px.
    minSize: 180,
  },
  {
    id: 'metadata',
    header: t(pipelineMessages.lastRun),
    cell: ({ row }) => renderSnippet(cells.lastRun, row.original),
    size: 200,
    minSize: 180,
  },
  {
    id: 'actions',
    header: t(pipelineMessages.actions),
    cell: ({ row }) => renderSnippet(cells.actions, row.original),
    size: 200,
    minSize: 80,
  },
];
