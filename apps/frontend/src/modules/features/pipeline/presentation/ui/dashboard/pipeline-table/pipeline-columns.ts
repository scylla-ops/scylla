import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { PipelineMetadata } from '../../../../domain/structs/pipeline.struct.ts';
import { pipelineMessages } from '../../../pipeline.messages.ts';

/** One snippet per column, each taking the row it renders. */
export interface PipelineCells {
  status: Snippet<[PipelineMetadata]>;
  history: Snippet<[PipelineMetadata]>;
  lastRun: Snippet<[PipelineMetadata]>;
  actions: Snippet<[PipelineMetadata]>;
}

/**
 * The pipelines table, as a column definition.
 *
 * `size` / `minSize` survived the port untouched — the same numbers the React
 * version declared, which is the dividend of TanStack Table being
 * framework-agnostic data. What changed is that a `cell` returns a Svelte
 * snippet instead of a React element, so the markup stays in `PipelineTable`
 * and this file stays a `.ts`.
 *
 * The cells take no `meta` bag either: the React version threaded six callbacks
 * and four flags through one, because a column definition was the only place a
 * cell could reach them. A snippet closes over the component that declares it.
 */
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
    // No size: takes every pixel the sized columns leave, down to 180px.
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
