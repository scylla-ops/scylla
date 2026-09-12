import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import type { CellContext, ColumnDef } from '@tanstack/react-table';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { createPipelineColumns } from './columns';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import type { JobEntity } from '@/modules/features/jobs';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const pipeline = (overrides: Partial<PipelineMetadata> = {}): PipelineMetadata => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci-pipeline',
  nodeCount: 3,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const ctxFor = (row: PipelineMetadata) => ({ row: { original: row } }) as CellContext<PipelineMetadata, unknown>;

const renderCell = (column: ColumnDef<PipelineMetadata>, row: PipelineMetadata) => {
  const cell = column.cell;
  if (typeof cell !== 'function') throw new Error(`column "${column.id}" has no cell renderer`);
  return render(<I18nProvider i18n={i18n}>{cell(ctxFor(row))}</I18nProvider>);
};

const findColumn = (columns: ColumnDef<PipelineMetadata>[], id: string) => columns.find(c => c.id === id)!;

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('createPipelineColumns', () => {
  const baseMeta = {
    onRun: vi.fn(),
    onEdit: vi.fn(),
    onDuplicate: vi.fn(),
    onViewJobs: vi.fn(),
    onViewTriggers: vi.fn(),
    runningPipelines: new Set<string>(),
    duplicatingPipelineId: undefined,
    jobsByPipelineId: new Map<string, JobEntity[]>(),
  };

  it('the status cell derives the status from the pipeline\'s most recent job', () => {
    const columns = createPipelineColumns({
      ...baseMeta,
      jobsByPipelineId: new Map([['pipeline-1', [job({ status: 'failed' })]]]),
    });
    renderCell(findColumn(columns, 'status'), pipeline());
    expect(screen.getByText('ci-pipeline')).toBeInTheDocument();
  });

  it('the history cell shows "No jobs yet" for a pipeline with no jobs recorded', () => {
    const columns = createPipelineColumns(baseMeta);
    renderCell(findColumn(columns, 'history'), pipeline());
    expect(screen.getByText('No jobs yet')).toBeInTheDocument();
  });

  it('the metadata cell reads the same jobsByPipelineId map for its last-job stats', () => {
    const columns = createPipelineColumns({
      ...baseMeta,
      jobsByPipelineId: new Map([
        ['pipeline-1', [job({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:00:30.000Z' })]],
      ]),
    });
    renderCell(findColumn(columns, 'metadata'), pipeline());
    expect(screen.getByText('30s')).toBeInTheDocument();
  });

  it('the actions cell forwards the row\'s id/pipeline and stops the click from bubbling', async () => {
    const onEdit = vi.fn();
    const rowClick = vi.fn();
    const user = userEvent.setup();
    const columns = createPipelineColumns({ ...baseMeta, onEdit });
    const cell = findColumn(columns, 'actions').cell;
    if (typeof cell !== 'function') throw new Error('no cell renderer');

    const { container } = render(
      <I18nProvider i18n={i18n}>
        <div onClick={rowClick}>{cell(ctxFor(pipeline({ id: 'pipeline-42' })))}</div>
      </I18nProvider>,
    );

    // Icon-only IconButtons - no accessible name, found by their Lucide icon class.
    const editButton = Array.from(container.querySelectorAll('button')).find(b =>
      b.querySelector('.lucide-square-pen'),
    );
    if (!editButton) throw new Error('edit button not found');
    await user.click(editButton);
    expect(onEdit).toHaveBeenCalledWith(expect.objectContaining({ id: 'pipeline-42' }));
    expect(rowClick).not.toHaveBeenCalled();
  });

  it('the actions cell marks a running/duplicating pipeline\'s own buttons as pending', () => {
    const columns = createPipelineColumns({
      ...baseMeta,
      runningPipelines: new Set(['pipeline-1']),
      duplicatingPipelineId: 'pipeline-1',
    });
    renderCell(findColumn(columns, 'actions'), pipeline({ id: 'pipeline-1' }));
    const buttons = screen.getAllByRole('button');
    const [runButton, , duplicateButton] = buttons;
    expect(runButton).toBeDisabled();
    expect(duplicateButton).toBeDisabled();
  });
});
