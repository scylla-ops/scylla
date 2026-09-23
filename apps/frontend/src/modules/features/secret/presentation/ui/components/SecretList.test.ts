import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { SecretEntity } from '../../../domain/entities/secret.entity.ts';
import type { SecretRepository } from '../../../domain/repository/secret.repository.ts';
import SecretList from './SecretList.svelte';

/**
 * What `secret-columns.test.tsx` covered, now through the table that renders
 * them: a cell is a snippet and a snippet has no existence outside its
 * component, so there is nothing left to call in isolation. The assertions are
 * the same ones.
 */

const toastSuccess = vi.fn();
vi.mock('svelte-sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

const secret = (overrides: Partial<SecretEntity> = {}): SecretEntity => ({
  id: 'secret-1',
  projectId: 'project-1',
  name: 'DATABASE_URL',
  description: 'prod db',
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

let teardown: Array<() => void> = [];
let deleteById: ReturnType<typeof vi.fn>;

const setUp = (secrets: SecretEntity[] = [secret()]) => {
  deleteById = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const repository = {
    listByProjectId: vi.fn().mockResolvedValue(ScyllaResult.success(secrets)),
    create: vi.fn(),
    deleteById,
  } as unknown as SecretRepository;
  const cache = withQueryClient();
  teardown = [cache.restore, withRegistry({ secret: { secretRepository: repository } })];

  return render(SecretList, { secrets, projectId: 'project-1' });
};

beforeEach(() => {
  toastSuccess.mockClear();
  useSelectionStore.setState({ selectedIds: {} });
});

afterEach(() => teardown.forEach(restore => restore()));

describe('SecretList', () => {
  it('shows a secret’s name and its copyable id, never a value', () => {
    setUp();

    expect(screen.getByText('DATABASE_URL')).toBeInTheDocument();
    expect(screen.getByText('secret-1')).toBeInTheDocument();
  });

  it('shows the description', () => {
    setUp([secret({ description: 'staging db' })]);

    expect(screen.getByText('staging db')).toBeInTheDocument();
  });

  it('formats the creation day', () => {
    setUp([secret({ createdAt: '2026-03-15T00:00:00.000Z' })]);

    // The exact wording is `formatDay`'s business; that the third column is not
    // blank is this test's.
    expect(screen.getAllByRole('cell')[2]).toHaveTextContent(/\S/);
  });

  it('selects a row when it is clicked', async () => {
    setUp();

    await userEvent.click(screen.getByText('DATABASE_URL'));

    expect(useSelectionStore.getState().selectedIds.secrets).toEqual(['secret-1']);
  });

  it('asks for confirmation before deleting, without selecting the row', async () => {
    setUp();

    await userEvent.click(screen.getByRole('button', { name: 'Delete secret' }));

    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    expect(deleteById).not.toHaveBeenCalled();
    // The click must not bubble to the row underneath.
    expect(useSelectionStore.getState().selectedIds.secrets ?? []).toEqual([]);
  });

  it('deletes and reports it once the confirmation is accepted', async () => {
    setUp([secret({ id: 'secret-42' })]);

    await userEvent.click(screen.getByRole('button', { name: 'Delete secret' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(deleteById).toHaveBeenCalledWith('secret-42'));
    await waitFor(() => expect(toastSuccess).toHaveBeenCalled());
  });

  it('renders the empty state when the project has no secret', () => {
    setUp([]);

    expect(screen.getByText('No results.')).toBeInTheDocument();
  });
});
