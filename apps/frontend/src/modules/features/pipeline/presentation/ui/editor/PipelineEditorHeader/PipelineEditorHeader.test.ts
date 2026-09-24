import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { render } from '@/test/render.svelte.ts';
import PipelineEditorHeaderFixture from './PipelineEditorHeader.fixture.svelte';

const onSubmit = vi.fn();

const grantOnly = (...permissions: Permission[]) =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

const renderHeader = (props: Record<string, unknown> = {}) =>
  render(PipelineEditorHeaderFixture, {
    onSubmit,
    submitLabel: 'Save',
    mode: 'edit',
    isDirty: false,
    ...props,
  });

beforeEach(() => vi.clearAllMocks());

describe('PipelineEditorHeader', () => {
  it('shows a draft for a pipeline that does not exist yet', () => {
    grantOnly(Permission.CREATE_PIPELINE);
    renderHeader({ mode: 'create', submitLabel: 'Create' });

    expect(screen.getByText('Draft — not created yet')).toBeInTheDocument();
  });

  it('shows unsaved changes, then saving, then saved', async () => {
    grantOnly(Permission.UPDATE_PIPELINE);
    const { rerender } = renderHeader({ isDirty: true });
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();

    await rerender({ isDirty: true, isSaving: true });
    expect(screen.getByText('Saving…')).toBeInTheDocument();

    await rerender({ isDirty: false, isSaving: false });
    expect(screen.getByText('All changes saved')).toBeInTheDocument();
  });

  it('submits when the user may edit the pipeline', async () => {
    grantOnly(Permission.UPDATE_PIPELINE);
    renderHeader({ isDirty: true });

    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalled();
  });

  it('disables the submit button for a user who may not edit the pipeline', () => {
    grantOnly();
    renderHeader({ isDirty: true });

    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('disables the blueprint tab when the script is invalid', () => {
    grantOnly(Permission.UPDATE_PIPELINE);
    renderHeader({ blueprintDisabled: true });

    expect(screen.getByRole('tab', { name: 'Blueprint' })).toBeDisabled();
  });
});
