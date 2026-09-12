import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import type { CellContext, ColumnDef } from '@tanstack/react-table';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { createJobColumns } from './columns';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const ctxFor = (row: JobEntity) => ({ row: { original: row } }) as CellContext<JobEntity, unknown>;

const renderCell = (column: ColumnDef<JobEntity>, row: JobEntity) => {
  const cell = column.cell;
  if (typeof cell !== 'function') throw new Error(`column "${column.id}" has no cell renderer`);
  return render(<I18nProvider i18n={i18n}>{cell(ctxFor(row))}</I18nProvider>);
};

const findColumn = (columns: ColumnDef<JobEntity>[], key: string) =>
  columns.find(c => (c as { accessorKey?: string }).accessorKey === key || c.id === key)!;

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('createJobColumns', () => {
  const meta = { pipelineId: 'pipeline-1', onDelete: vi.fn(), onView: vi.fn(), onOpenJobLog: vi.fn() };
  const columns = createJobColumns(meta);

  it('the duration cell shows a dash for a job that never started', () => {
    renderCell(findColumn(columns, 'duration'), job({ startedAt: undefined, finishedAt: undefined }));
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('the duration cell computes an elapsed time for a finished job', () => {
    renderCell(
      findColumn(columns, 'duration'),
      job({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:01:05.000Z' }),
    );
    expect(screen.getByText('1m 5s')).toBeInTheDocument();
  });

  it('the duration cell measures a still-running job against now', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:30.000Z'));

    renderCell(
      findColumn(columns, 'duration'),
      job({ status: 'running', startedAt: '2026-01-01T00:00:00.000Z', finishedAt: undefined }),
    );
    expect(screen.getByText('30s')).toBeInTheDocument();

    vi.useRealTimers();
  });

  it('the id cell shows the (truncated) job id', () => {
    renderCell(findColumn(columns, 'id'), job({ id: 'job-abcdefghijklmnop' }));
    expect(screen.getByText('job-abcdefgh...')).toBeInTheDocument();
  });

  it('the created cell shows a relative time', () => {
    renderCell(findColumn(columns, 'createdAt'), job({ createdAt: '2026-01-01T00:00:00.000Z' }));
    expect(screen.getByText(/\S/)).toBeInTheDocument();
  });

  it('the actions cell stops the row click and forwards the id to each handler', async () => {
    const onView = vi.fn();
    const onDelete = vi.fn();
    const onOpenJobLog = vi.fn();
    const rowClick = vi.fn();
    const actionColumns = createJobColumns({ ...meta, onView, onDelete, onOpenJobLog });
    const cell = findColumn(actionColumns, 'actions').cell;
    if (typeof cell !== 'function') throw new Error('no cell renderer');

    const user = userEvent.setup();
    render(
      <I18nProvider i18n={i18n}>
        <div onClick={rowClick}>{cell(ctxFor(job({ id: 'job-9' })))}</div>
      </I18nProvider>,
    );

    const [viewButton] = screen.getAllByRole('button');
    await user.click(viewButton);

    expect(onView).toHaveBeenCalledWith('job-9');
    expect(rowClick).not.toHaveBeenCalled();
  });
});
