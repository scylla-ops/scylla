import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { contextStore } from '@platform/context';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ProjectEntity } from '../../../domain/entities/project.entity.ts';
import type { ProjectRepository } from '../../../domain/repository/project.repository.ts';
import AddProjectDialog from '../AddProjectDialog.svelte';
import EditProjectDialog from '../EditProjectDialog.svelte';

const toastError = vi.fn();
vi.mock('@shared/presentation/utils/toast.ts', () => ({
  toast: { error: (...args: unknown[]) => toastError(...args), success: vi.fn() },
}));

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'web',
  description: 'the web app',
  ...overrides,
});

let teardown: Array<() => void> = [];
let create: ReturnType<typeof vi.fn>;
let update: ReturnType<typeof vi.fn>;

const install = () => {
  create = vi.fn().mockResolvedValue(ScyllaResult.success(project()));
  update = vi.fn().mockResolvedValue(ScyllaResult.success(project()));
  const repository = {
    getByOrganizationId: vi.fn(),
    listMembers: vi.fn(),
    create,
    update,
    delete: vi.fn(),
  } as unknown as ProjectRepository;

  const cache = withQueryClient();
  teardown = [cache.restore, withRegistry({ project: { projectRepository: repository } })];
};

beforeEach(() => {
  vi.clearAllMocks();
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme' } });
});

afterEach(() => teardown.forEach(restore => restore()));

describe('AddProjectDialog', () => {
  it('toasts an error and never calls the repository when no organization is selected', async () => {
    contextStore.setState({ organization: { id: null, name: null } });
    install();
    render(AddProjectDialog, { open: true, setOpen: vi.fn() });
    await focusSettled();

    // Description isn't marked optional either, so both fields are needed just
    // to enable Create — the organization check only runs once the form itself
    // considers the input valid.
    await userEvent.type(screen.getByLabelText('Project name'), 'web');
    await userEvent.type(screen.getByLabelText('Description'), 'the web app');
    await userEvent.click(screen.getByRole('button', { name: 'Create Project' }));

    expect(toastError).toHaveBeenCalledWith(
      'Project name is required and you must select an organization.',
    );
    expect(create).not.toHaveBeenCalled();
  });

  it('creates the project under the current organization and closes on success', async () => {
    const setOpen = vi.fn();
    install();
    render(AddProjectDialog, { open: true, setOpen });
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Project name'), 'web');
    await userEvent.type(screen.getByLabelText('Description'), '  the web app  ');
    await userEvent.click(screen.getByRole('button', { name: 'Create Project' }));

    await waitFor(() => expect(create).toHaveBeenCalledWith('web', 'org-1', 'the web app'));
    await waitFor(() => expect(setOpen).toHaveBeenCalledWith(false));
  });

  it("a whitespace-only description fails the form's own validity check, so Create never enables", async () => {
    install();
    render(AddProjectDialog, { open: true, setOpen: vi.fn() });
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Project name'), 'web');
    await userEvent.type(screen.getByLabelText('Description'), '   ');

    expect(screen.getByRole('button', { name: 'Create Project' })).toBeDisabled();
  });
});

describe('EditProjectDialog', () => {
  it('prefills the current name and description, ready to save unmodified', async () => {
    install();
    render(EditProjectDialog, { open: true, setOpen: vi.fn(), project: project() });
    await focusSettled();

    expect(screen.getByLabelText('Project name')).toHaveValue('web');
    expect(screen.getByLabelText('Description')).toHaveValue('the web app');
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
  });

  it('falls back to a generic placeholder, and a blank prefill, when there is no description', async () => {
    install();
    render(EditProjectDialog, {
      open: true,
      setOpen: vi.fn(),
      project: project({ description: '' }),
    });
    await focusSettled();

    expect(screen.getByLabelText('Description')).toHaveValue('');
    expect(screen.getByLabelText('Description')).toHaveAttribute(
      'placeholder',
      'Add a description...',
    );
  });

  it('submits the trimmed changes for this project id and closes on success', async () => {
    const setOpen = vi.fn();
    install();
    render(EditProjectDialog, {
      open: true,
      setOpen,
      project: project({ id: 'project-9' }),
    });
    await focusSettled();

    const nameInput = screen.getByLabelText('Project name');
    await userEvent.clear(nameInput);
    await userEvent.type(nameInput, '  renamed  ');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() =>
      expect(update).toHaveBeenCalledWith('project-9', 'renamed', 'the web app'),
    );
    await waitFor(() => expect(setOpen).toHaveBeenCalledWith(false));
  });
});
