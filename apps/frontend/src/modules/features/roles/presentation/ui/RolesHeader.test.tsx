import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { RolesHeader } from './RolesHeader';

describe('RolesHeader', () => {
  it('shows the role count', () => {
    renderWithI18n(<RolesHeader count={3} />);
    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByText('Roles')).toBeInTheDocument();
  });

  it('hides "Create role" entirely without onNew', () => {
    renderWithI18n(<RolesHeader count={0} />);
    expect(screen.queryByRole('button', { name: 'Create role' })).not.toBeInTheDocument();
  });

  it('"Create role" calls onNew when given', async () => {
    const onNew = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RolesHeader count={0} onNew={onNew} />);
    await user.click(screen.getByRole('button', { name: 'Create role' }));
    expect(onNew).toHaveBeenCalled();
  });

  it('forwards the selection props through to FeatureHeader', async () => {
    const onSelectAll = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RolesHeader count={5} onSelectAll={onSelectAll} />);
    await user.click(screen.getByRole('button', { name: 'Select all' }));
    expect(onSelectAll).toHaveBeenCalled();
  });
});
