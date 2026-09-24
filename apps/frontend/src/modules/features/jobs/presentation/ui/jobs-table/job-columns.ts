import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { JobEntity } from '../../../domain/entities/job.entity.ts';
import { jobsMessages } from '../jobs.messages.ts';

export interface JobCells {
  status: Snippet<[JobEntity]>;
  id: Snippet<[JobEntity]>;
  timeline: Snippet<[JobEntity]>;
  duration: Snippet<[JobEntity]>;
  created: Snippet<[JobEntity]>;
  actions: Snippet<[JobEntity]>;
}

export const jobColumns = (cells: JobCells): DataTableColumn<JobEntity>[] => [
  {
    accessorKey: 'status',
    header: t(jobsMessages.status),
    cell: ({ row }) => renderSnippet(cells.status, row.original),
    size: 200,
    minSize: 150,
  },
  {
    accessorKey: 'id',
    header: t(jobsMessages.jobId),
    cell: ({ row }) => renderSnippet(cells.id, row.original),
    size: 200,
    minSize: 180,
  },
  {
    id: 'timeline',
    header: t(jobsMessages.timeline),
    cell: ({ row }) => renderSnippet(cells.timeline, row.original),
    // No size: takes the space left, down to 200px.
    minSize: 200,
  },
  {
    id: 'duration',
    header: t(jobsMessages.duration),
    cell: ({ row }) => renderSnippet(cells.duration, row.original),
    size: 100,
    minSize: 80,
  },
  {
    accessorKey: 'createdAt',
    header: t(jobsMessages.created),
    cell: ({ row }) => renderSnippet(cells.created, row.original),
    size: 100,
    minSize: 80,
  },
  {
    id: 'actions',
    header: t(jobsMessages.actions),
    cell: ({ row }) => renderSnippet(cells.actions, row.original),
    minSize: 100,
    size: 130,
  },
];
