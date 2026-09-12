import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AddRoleSelect } from './AddRoleSelect';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

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
