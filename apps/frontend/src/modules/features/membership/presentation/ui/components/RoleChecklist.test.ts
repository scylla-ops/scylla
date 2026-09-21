import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
import type { AssignableRole } from '../../assignable-roles.state.svelte.ts';
import RoleChecklist from './RoleChecklist.svelte';

const assignable = (overrides: Partial<AssignableRole> = {}): AssignableRole => ({
  roleId: 'project-viewer',
  name: 'Project viewer',
  description: 'Read-only access',
  ...overrides,
});

const props = (overrides: Record<string, unknown> = {}) => ({
  label: 'Roles',
  roles: [assignable()],
  selected: new Set<string>(),
  onToggle: vi.fn(),
  ...overrides,
});

describe('RoleChecklist', () => {
  it('says it is loading rather than claiming there is nothing to grant', () => {
    render(RoleChecklist, props({ roles: [], isLoading: true }));

    expect(screen.getByText('Loading…')).toBeInTheDocument();
    expect(screen.queryByText('No role can be granted here.')).not.toBeInTheDocument();
  });

  it('says so when no role can be granted at this scope', () => {
    render(RoleChecklist, props({ roles: [] }));
    expect(screen.getByText('No role can be granted here.')).toBeInTheDocument();
  });

  it('lists each role with its description', () => {
    render(RoleChecklist, props());

    expect(screen.getByText('Project viewer')).toBeInTheDocument();
    expect(screen.getByText('Read-only access')).toBeInTheDocument();
  });

  it('reports the role id that was ticked', async () => {
    const onToggle = vi.fn();
    render(RoleChecklist, props({ onToggle }));

    await userEvent.click(screen.getByRole('checkbox'));

    expect(onToggle).toHaveBeenCalledWith('project-viewer');
  });

  it('shows a ticked box for a role already in the selection', () => {
    render(RoleChecklist, props({ selected: new Set(['project-viewer']) }));

    expect(screen.getByRole('checkbox')).toBeChecked();
  });

  it('allows several roles at once — access is the sum of them, not one of them', async () => {
    const onToggle = vi.fn();
    render(
      RoleChecklist,
      props({
        roles: [assignable(), assignable({ roleId: 'project-admin', name: 'Project admin' })],
        selected: new Set(['project-viewer']),
        onToggle,
      }),
    );

    const boxes = screen.getAllByRole('checkbox');
    expect(boxes[0]).toBeChecked();

    await userEvent.click(boxes[1]);
    expect(onToggle).toHaveBeenCalledWith('project-admin');
  });

  it('shows a role the member already holds as ticked, locked and labelled', async () => {
    const onToggle = vi.fn();
    render(
      RoleChecklist,
      props({ alreadyHeld: new Set(['project-viewer']), onToggle }),
    );

    const box = screen.getByRole('checkbox');
    expect(box).toBeChecked();
    expect(box).toBeDisabled();
    expect(screen.getByText('Held')).toBeInTheDocument();

    await userEvent.click(box);
    expect(onToggle).not.toHaveBeenCalled();
  });

  it('disables every box while a write is in flight', () => {
    render(RoleChecklist, props({ disabled: true }));

    expect(screen.getByRole('checkbox')).toBeDisabled();
  });
});
