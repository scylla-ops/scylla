import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { contextStore } from '@platform/context';
import { permissionsStore, PermissionScope } from '@platform/authz';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ProjectEntity } from '../../../domain/entities/project.entity.ts';
import type { ProjectRepository } from '../../../domain/repository/project.repository.ts';
import ProjectCard from './ProjectCard.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn() } }));

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'web',
  description: 'the web app',
  ...overrides,
});

let teardown: Array<() => void> = [];
let testNavigator: ReturnType<typeof installTestNavigator>;

const renderCard = (entity: ProjectEntity = project()) => {
  const repository = {
    getByOrganizationId: vi.fn(),
    listMembers: vi.fn(),
    create: vi.fn(),
    update: vi.fn().mockResolvedValue(ScyllaResult.success(entity)),
    delete: vi.fn(),
  } as unknown as ProjectRepository;

  const cache = withQueryClient();
  const restoreRegistry = withRegistry({ project: { projectRepository: repository } });
  testNavigator = installTestNavigator({ pathname: '/acme/projects' });
  teardown = [cache.restore, restoreRegistry, testNavigator.restore];

  return render(ProjectCard, { project: entity });
};

beforeEach(() => {
  selectionStore.setState({ selectedIds: {} });
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });
});

afterEach(() => teardown.forEach(restore => restore()));

describe('ProjectCard', () => {
  it('navigates to the project when clicked', async () => {
    renderCard();

    await userEvent.click(screen.getByText('web'));

    expect(testNavigator.navigate).toHaveBeenCalled();
  });

  it('shows the description', () => {
    renderCard(project({ description: 'a real one' }));
    expect(screen.getByText('a real one')).toBeInTheDocument();
  });

  it('shows an italic "No description" placeholder when there is none', () => {
    renderCard(project({ description: '' }));
    expect(screen.getByText('No description')).toBeInTheDocument();
  });

  // IconButton renders its tooltip text visually-hidden inside the button, so
  // the Edit affordance carries a real accessible name.
  const findEditButton = () => screen.queryByRole('button', { name: 'Edit' });

  it('the Edit button is hidden without UPDATE_PROJECT on this project', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    renderCard();
    expect(findEditButton()).toBeNull();
  });

  it('the Edit button opens the edit dialog without navigating', async () => {
    renderCard();

    await userEvent.click(findEditButton()!);

    expect(testNavigator.navigate).not.toHaveBeenCalled();
    expect(await screen.findByText('Edit project')).toBeInTheDocument();
  });

  it('checking the selection checkbox toggles selection without navigating', async () => {
    renderCard(project({ id: 'project-9' }));

    await userEvent.click(screen.getByRole('checkbox'));

    expect(testNavigator.navigate).not.toHaveBeenCalled();
    expect(selectionStore.getState().selectedIds.projects).toContain('project-9');
  });
});
