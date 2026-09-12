import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import type { CellContext, ColumnDef } from '@tanstack/react-table';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { createTriggerColumns } from './columns';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 't1',
  pipelineId: 'p1',
  name: 'nightly-build',
  source: { kind: TriggerKind.Cron, expression: '0 9 * * *' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const ctxFor = (row: TriggerEntity) => ({ row: { original: row } }) as CellContext<TriggerEntity, unknown>;

const renderCell = (column: ColumnDef<TriggerEntity>, row: TriggerEntity) => {
  const cell = column.cell;
  if (typeof cell !== 'function') throw new Error(`column "${column.id}" has no cell renderer`);
  return render(<I18nProvider i18n={i18n}>{cell(ctxFor(row))}</I18nProvider>);
};

const findColumn = (columns: ColumnDef<TriggerEntity>[], id: string) => columns.find(c => c.id === id)!;

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('createTriggerColumns', () => {
  const meta = {
    onFire: vi.fn(),
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onToggleEnabled: vi.fn(),
    firingIds: new Set<string>(),
  };
  const columns = createTriggerColumns(meta);

  it('the name cell shows the trigger name and its id', () => {
    renderCell(findColumn(columns, 'name'), trigger({ name: 'nightly-build', id: 't-9' }));
    expect(screen.getByText('nightly-build')).toBeInTheDocument();
    expect(screen.getByText('ID: t-9')).toBeInTheDocument();
  });

  it('the enabled cell toggles the switch and stops the row click from also firing', async () => {
    const onToggleEnabled = vi.fn();
    const rowClick = vi.fn();
    const user = userEvent.setup();
    const toggleColumns = createTriggerColumns({ ...meta, onToggleEnabled });
    const cell = findColumn(toggleColumns, 'enabled').cell;
    if (typeof cell !== 'function') throw new Error('no cell renderer');

    render(
      <I18nProvider i18n={i18n}>
        <div onClick={rowClick}>{cell(ctxFor(trigger({ enabled: false })))}</div>
      </I18nProvider>,
    );

    await user.click(screen.getByRole('switch'));
    expect(onToggleEnabled).toHaveBeenCalledWith(expect.objectContaining({ id: 't1' }), true);
    expect(rowClick).not.toHaveBeenCalled();
  });

  it('the enabled switch reflects the trigger\'s current state', () => {
    renderCell(findColumn(columns, 'enabled'), trigger({ enabled: true }));
    expect(screen.getByRole('switch')).toBeChecked();
  });

  it('the actions cell forwards the row\'s id/trigger and stops the click from bubbling', async () => {
    const onDelete = vi.fn();
    const rowClick = vi.fn();
    const user = userEvent.setup();
    const actionColumns = createTriggerColumns({ ...meta, onDelete });
    const cell = findColumn(actionColumns, 'actions').cell;
    if (typeof cell !== 'function') throw new Error('no cell renderer');

    render(
      <I18nProvider i18n={i18n}>
        <div onClick={rowClick}>{cell(ctxFor(trigger({ id: 't-42' })))}</div>
      </I18nProvider>,
    );

    const buttons = screen.getAllByRole('button');
    const deleteButton = buttons[buttons.length - 1];
    await user.click(deleteButton);

    expect(onDelete).toHaveBeenCalledWith('t-42');
    expect(rowClick).not.toHaveBeenCalled();
  });

  it('marks a firing trigger\'s row as pending in the actions cell', () => {
    const firingColumns = createTriggerColumns({ ...meta, firingIds: new Set(['t-firing']) });
    const { container } = renderCell(findColumn(firingColumns, 'actions'), trigger({ id: 't-firing' }));
    const [fireButton] = screen.getAllByRole('button');
    expect(fireButton).toBeDisabled();
    expect(container.querySelector('.animate-spin')).toBeInTheDocument();
  });
});
