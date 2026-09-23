import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { OrganizationRepository } from '../../domain/repository/organization.repository.ts';
import EditOrganizationDialog from './EditOrganizationDialog.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn() } }));

let teardown: Array<() => void> = [];
let update: ReturnType<typeof vi.fn>;

const setUp = (
  organization: { id: string; name: string; description?: string },
  setOpen: (open: boolean) => void = vi.fn(),
) => {
  update = vi.fn().mockResolvedValue(ScyllaResult.success({ ...organization }));
  const repository = { update } as unknown as OrganizationRepository;

  const cache = withQueryClient();
  teardown = [
    cache.restore,
    withRegistry({ organization: { organizationRepository: repository } }),
  ];

  return render(EditOrganizationDialog, { open: true, setOpen, organization });
};

beforeEach(() => vi.clearAllMocks());
afterEach(() => teardown.forEach(restore => restore()));

describe('EditOrganizationDialog', () => {
  it('starts with blank fields — the current values are placeholders — so Save starts disabled', async () => {
    setUp({ id: 'org-1', name: 'Acme Corp', description: 'a real one' });
    await focusSettled();

    expect(screen.getByLabelText('Organization name')).toHaveValue('');
    expect(screen.getByLabelText('Organization name')).toHaveAttribute('placeholder', 'Acme Corp');
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('falls back to a generic placeholder when there is no existing description', async () => {
    setUp({ id: 'org-1', name: 'Acme' });
    await focusSettled();

    expect(screen.getByLabelText('Description')).toHaveAttribute(
      'placeholder',
      'Add a description...',
    );
  });

  it('submits the trimmed new name and description for this id, and closes on success', async () => {
    const setOpen = vi.fn();
    const user = userEvent.setup();
    setUp({ id: 'org-9', name: 'Acme Corp' }, setOpen);
    await focusSettled();

    await user.type(screen.getByLabelText('Organization name'), '  New Name  ');
    await user.type(screen.getByLabelText('Description'), '  new desc  ');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() => expect(update).toHaveBeenCalledWith('org-9', 'New Name', 'new desc'));
    await waitFor(() => expect(setOpen).toHaveBeenCalledWith(false));
  });
});
