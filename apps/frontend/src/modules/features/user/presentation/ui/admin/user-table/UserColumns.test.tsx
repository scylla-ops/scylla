import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import type { CellContext, ColumnDef } from '@tanstack/react-table';
import { createUserColumns } from './UserColumns';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const user = (overrides: Partial<UserEntity> = {}): UserEntity => ({
  userId: 'user-1',
  username: 'ravenne',
  createdAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const ctxFor = (row: UserEntity) => ({ row: { original: row } }) as CellContext<UserEntity, unknown>;

const renderCell = (column: ColumnDef<UserEntity>, row: UserEntity) => {
  const cell = column.cell;
  if (typeof cell !== 'function') throw new Error(`column "${column.id}" has no cell renderer`);
  return render(<I18nProvider i18n={i18n}>{cell(ctxFor(row))}</I18nProvider>);
};

const findColumn = (columns: ColumnDef<UserEntity>[], id: string) => columns.find(c => c.id === id)!;

describe('createUserColumns', () => {
  it('the username cell shows the first letter as an avatar fallback and the full username', () => {
    const columns = createUserColumns({ onView: vi.fn() });
    renderCell(findColumn(columns, 'username'), user({ username: 'ravenne' }));

    expect(screen.getByText('R')).toBeInTheDocument();
    expect(screen.getByText('ravenne')).toBeInTheDocument();
  });

  it('the creationDate cell formats the date', () => {
    const columns = createUserColumns({ onView: vi.fn() });
    renderCell(findColumn(columns, 'creationDate'), user({ createdAt: '2026-03-15T00:00:00.000Z' }));
    expect(screen.getByText(/\S/)).toBeInTheDocument();
  });

  it('the actions cell calls onView with the row\'s userId', async () => {
    const onView = vi.fn();
    const columns = createUserColumns({ onView });
    const userEventInstance = userEvent.setup();

    renderCell(findColumn(columns, 'actions'), user({ userId: 'user-42' }));
    await userEventInstance.click(screen.getByRole('button'));

    expect(onView).toHaveBeenCalledWith('user-42');
  });
});
