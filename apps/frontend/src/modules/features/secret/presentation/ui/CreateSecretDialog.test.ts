import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { SecretRepository } from '../../domain/repository/secret.repository.ts';
import CreateSecretDialog from './CreateSecretDialog.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn() } }));

let teardown: Array<() => void> = [];
let create: ReturnType<typeof vi.fn>;

const setUp = (props: { projectId?: string; setOpen?: (open: boolean) => void } = {}) => {
  create = vi.fn().mockResolvedValue(
    ScyllaResult.success({
      id: 'secret-1',
      projectId: props.projectId ?? 'project-1',
      name: 'DATABASE_URL',
      description: '',
      createdAt: '',
      updatedAt: '',
    }),
  );
  const repository = {
    listByProjectId: vi.fn(),
    create,
    deleteById: vi.fn(),
  } as unknown as SecretRepository;

  const cache = withQueryClient();
  teardown = [cache.restore, withRegistry({ secret: { secretRepository: repository } })];

  return render(CreateSecretDialog, {
    projectId: props.projectId ?? 'project-1',
    isOpen: true,
    setOpen: props.setOpen ?? vi.fn(),
  });
};

afterEach(() => teardown.forEach(restore => restore()));
beforeEach(() => vi.clearAllMocks());

const fillValidForm = async (user: ReturnType<typeof userEvent.setup>) => {
  await focusSettled();
  await user.type(screen.getByLabelText('Secret name'), 'DATABASE_URL');
  await user.type(screen.getByLabelText('Description'), 'the prod db');
  await user.type(screen.getByLabelText('Value'), 'postgres://...');
};

describe('CreateSecretDialog', () => {
  it('rejects a name with characters outside the backend pattern, keeping Create disabled', async () => {
    const user = userEvent.setup();
    setUp();

    await focusSettled();
    await user.type(screen.getByLabelText('Secret name'), 'not a valid name');
    await user.type(screen.getByLabelText('Description'), 'desc');
    await user.type(screen.getByLabelText('Value'), 'value');

    expect(screen.getByRole('button', { name: 'Create' })).toBeDisabled();
  });

  it('creates the secret scoped to the given project and closes the dialog immediately', async () => {
    const setOpen = vi.fn();
    const user = userEvent.setup();
    setUp({ projectId: 'project-42', setOpen });

    await fillValidForm(user);
    await user.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() =>
      expect(create).toHaveBeenCalledWith({
        projectId: 'project-42',
        name: 'DATABASE_URL',
        description: 'the prod db',
        value: 'postgres://...',
      }),
    );
    // Unlike the other create dialogs, this one closes right away rather than
    // waiting on the mutation's own onSuccess — it never passes one.
    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
