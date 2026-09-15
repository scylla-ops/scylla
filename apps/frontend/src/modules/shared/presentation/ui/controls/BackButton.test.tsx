import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BackButton } from './BackButton';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

describe('BackButton', () => {
  it('navigates back (-1) by default when clicked', async () => {
    const user = userEvent.setup();
    render(<BackButton />);
    await user.click(screen.getByRole('button'));
    expect(navigateMock).toHaveBeenCalledWith(-1);
  });

  it('calls a given onClick instead of navigating', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<BackButton onClick={onClick} />);
    await user.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalled();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('shows the "Back" label by default', () => {
    render(<BackButton />);
    expect(screen.getByRole('button', { name: /back/i })).toBeInTheDocument();
  });

  it('shows a custom label', () => {
    render(<BackButton label='Return to list' />);
    expect(screen.getByRole('button', { name: 'Return to list' })).toBeInTheDocument();
  });

  it('hides the label entirely in iconOnly mode', () => {
    render(<BackButton iconOnly label='Return to list' />);
    expect(screen.queryByText('Return to list')).not.toBeInTheDocument();
  });
});
