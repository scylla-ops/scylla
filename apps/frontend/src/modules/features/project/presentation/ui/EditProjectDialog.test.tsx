import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { EditProjectDialog } from './EditProjectDialog';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';

const mutateMock = vi.fn();
vi.mock('@/modules/features/project/presentation/hooks/use-update-project.ts', () => ({
  useUpdateProject: () => ({ mutate: mutateMock, isPending: false }),
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'web',
  description: 'the web app',
  ...overrides,
});

beforeEach(() => {
  mutateMock.mockReset();
});

describe('EditProjectDialog', () => {
  it('prefills the current name and description, ready to save unmodified', () => {
    renderWithI18n(<EditProjectDialog open setOpen={vi.fn()} project={project()} />);
    expect(screen.getByLabelText('Project name')).toHaveValue('web');
    expect(screen.getByLabelText('Description')).toHaveValue('the web app');
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
  });

  it('falls back to a generic description placeholder, and a blank prefill, when the project has none', () => {
    renderWithI18n(<EditProjectDialog open setOpen={vi.fn()} project={project({ description: '' })} />);
    expect(screen.getByLabelText('Description')).toHaveValue('');
    expect(screen.getByLabelText('Description')).toHaveAttribute('placeholder', 'Add a description...');
  });

  it('submits the trimmed changes for this project id and closes on success', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess());
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<EditProjectDialog open setOpen={setOpen} project={project({ id: 'project-9' })} />);

    const nameInput = screen.getByLabelText('Project name');
    await user.clear(nameInput);
    await user.type(nameInput, '  renamed  ');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(mutateMock).toHaveBeenCalledWith(
      { projectId: 'project-9', name: 'renamed', description: 'the web app' },
      expect.any(Object),
    );
    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
