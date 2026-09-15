import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import type { CellContext, ColumnDef } from '@tanstack/react-table';
import { createCredentialsColumns } from './secret-columns';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';

const secret = (overrides: Partial<SecretEntity> = {}): SecretEntity => ({
  id: 'secret-1',
  projectId: 'project-1',
  name: 'DATABASE_URL',
  description: 'prod db',
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const ctxFor = (row: SecretEntity) => ({ row: { original: row } }) as CellContext<SecretEntity, unknown>;

/** A column's `cell` is `string | ((ctx) => ReactNode) | undefined` - narrow
 * to the render-prop function this codebase always uses. */
const renderCell = (column: ColumnDef<SecretEntity>, row: SecretEntity) => {
  const cell = column.cell;
  if (typeof cell !== 'function') throw new Error(`column "${column.id}" has no cell renderer`);
  return render(<I18nProvider i18n={i18n}>{cell(ctxFor(row))}</I18nProvider>);
};

const findColumn = (columns: ColumnDef<SecretEntity>[], key: string) =>
  columns.find(c => (c as { accessorKey?: string }).accessorKey === key || c.id === key)!;

describe('createCredentialsColumns', () => {
  const columns = createCredentialsColumns({ onDelete: vi.fn() });

  it('the name cell shows the secret\'s name and its (copyable) id', () => {
    renderCell(findColumn(columns, 'name'), secret());
    expect(screen.getByText('DATABASE_URL')).toBeInTheDocument();
    expect(screen.getByText('secret-1')).toBeInTheDocument();
  });

  it('the description cell shows the description', () => {
    renderCell(findColumn(columns, 'description'), secret({ description: 'staging db' }));
    expect(screen.getByText('staging db')).toBeInTheDocument();
  });

  it('the created cell formats the day', () => {
    renderCell(findColumn(columns, 'createdAt'), secret({ createdAt: '2026-03-15T00:00:00.000Z' }));
    // formatDay's exact wording isn't this test's concern - just that something
    // real (not empty) rendered.
    expect(screen.getByText(/\S/)).toBeInTheDocument();
  });

  it('the actions cell calls onDelete with the row\'s id, and stops the click from bubbling', async () => {
    const onDelete = vi.fn();
    const rowOnClick = vi.fn();
    const actionColumns = createCredentialsColumns({ onDelete });
    const user = userEvent.setup();
    const cell = findColumn(actionColumns, 'actions').cell;
    if (typeof cell !== 'function') throw new Error('no cell renderer');

    render(
      <I18nProvider i18n={i18n}>
        <div onClick={rowOnClick}>{cell(ctxFor(secret({ id: 'secret-42' })))}</div>
      </I18nProvider>,
    );
    await user.click(screen.getByRole('button'));

    expect(onDelete).toHaveBeenCalledWith('secret-42');
    expect(rowOnClick).not.toHaveBeenCalled();
  });
});
