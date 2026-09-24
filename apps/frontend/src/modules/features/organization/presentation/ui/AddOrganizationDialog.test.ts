import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { contextStore } from '@platform/context';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { OrganizationRepository } from '../../domain/repository/organization.repository.ts';
import AddOrganizationDialog from './AddOrganizationDialog.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn() } }));

let teardown: Array<() => void> = [];
let testNavigator: ReturnType<typeof installTestNavigator>;
let create: ReturnType<typeof vi.fn>;

const setUp = (props: { setOpen?: (open: boolean) => void; hideCancel?: boolean } = {}) => {
  create = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ id: 'org-1', name: 'Acme Corp', description: '' }));
  const repository = { create } as unknown as OrganizationRepository;

  const cache = withQueryClient();
  const restoreRegistry = withRegistry({ organization: { organizationRepository: repository } });
  testNavigator = installTestNavigator();
  teardown = [cache.restore, restoreRegistry, testNavigator.restore];

  return render(AddOrganizationDialog, {
    open: true,
    setOpen: props.setOpen ?? vi.fn(),
    hideCancel: props.hideCancel,
  });
};

beforeEach(() => {
  vi.clearAllMocks();
  contextStore.setState({ organization: { id: null, name: null } });
});

afterEach(() => teardown.forEach(restore => restore()));

describe('AddOrganizationDialog', () => {
  it('the Create button stays disabled until both name and description are entered', async () => {
    const user = userEvent.setup();
    setUp();
    await focusSettled();

    const button = screen.getByRole('button', { name: 'Create Organization' });
    expect(button).toBeDisabled();

    // Description isn't marked optional on this form either, even though the
    // dialog's own submit guard only actually cares about the name.
    await user.type(screen.getByLabelText('Organization name'), 'Acme Corp');
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Description'), 'a real one');
    expect(button).toBeEnabled();
  });

  it('submits the raw (untrimmed) name, then closes and navigates on success', async () => {
    const setOpen = vi.fn();
    const user = userEvent.setup();
    setUp({ setOpen });
    await focusSettled();

    await user.type(screen.getByLabelText('Organization name'), '  Acme Corp  ');
    await user.type(screen.getByLabelText('Description'), 'a real one');
    await user.click(screen.getByRole('button', { name: 'Create Organization' }));

    // Unlike the description, the name reaches the repository exactly as typed —
    // only the submit guard and the form's validity check trim a copy.
    await waitFor(() => expect(create).toHaveBeenCalledWith('  Acme Corp  ', 'a real one'));
    await waitFor(() => expect(setOpen).toHaveBeenCalledWith(false));
    expect(contextStore.getState().organization).toEqual({ id: 'org-1', name: 'Acme Corp' });
    expect(testNavigator.navigate).toHaveBeenCalledWith('/acme-corp/projects', undefined);
  });

  it('sends a trimmed, non-empty description', async () => {
    const user = userEvent.setup();
    setUp();
    await focusSettled();

    await user.type(screen.getByLabelText('Organization name'), 'Acme');
    await user.type(screen.getByLabelText('Description'), '  a real one  ');
    await user.click(screen.getByRole('button', { name: 'Create Organization' }));

    await waitFor(() => expect(create).toHaveBeenCalledWith('Acme', 'a real one'));
  });

  it('hides the Cancel button when hideCancel is set', async () => {
    setUp({ hideCancel: true });
    await focusSettled();

    expect(screen.queryByRole('button', { name: 'Cancel' })).not.toBeInTheDocument();
  });
});
