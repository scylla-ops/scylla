import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { AddRoleSelect } from './AddRoleSelect';

describe('AddRoleSelect', () => {
  it('renders nothing once there is no role left to add', () => {
    const { container } = renderWithI18n(<AddRoleSelect roles={[]} disabled={false} onSelect={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('shows a "+ role" placeholder and every offered role', async () => {
    const onSelect = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <AddRoleSelect
        roles={[
          { roleId: 'r1', name: 'Developer' },
          { roleId: 'r2', name: 'Viewer' },
        ]}
        disabled={false}
        onSelect={onSelect}
      />,
    );

    expect(screen.getByText('+ role')).toBeInTheDocument();
    await user.click(screen.getByRole('combobox'));
    await user.click(await screen.findByText('Viewer'));
    expect(onSelect).toHaveBeenCalledWith('r2');
  });

  it('is disabled when asked to be', () => {
    renderWithI18n(<AddRoleSelect roles={[{ roleId: 'r1', name: 'Developer' }]} disabled onSelect={vi.fn()} />);
    expect(screen.getByRole('combobox')).toBeDisabled();
  });
});
