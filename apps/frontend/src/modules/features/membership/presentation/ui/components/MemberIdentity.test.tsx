import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemberIdentity } from './MemberIdentity';

describe('MemberIdentity', () => {
  it("shows the member's name, with the raw first character as the avatar", () => {
    render(<MemberIdentity name='ravenne' />);
    expect(screen.getByText('ravenne')).toBeInTheDocument();
    // Rendered as-is; only a CSS class visually uppercases it.
    expect(screen.getByText('r', { selector: 'span[aria-hidden]' })).toBeInTheDocument();
  });

  it('carries the full name as a title, for a truncated long one', () => {
    render(<MemberIdentity name='a-very-long-username-that-truncates' />);
    expect(screen.getByText('a-very-long-username-that-truncates')).toHaveAttribute(
      'title',
      'a-very-long-username-that-truncates',
    );
  });

  it('renders a subtitle when given one', () => {
    render(<MemberIdentity name='ravenne' subtitle='2 roles' />);
    expect(screen.getByText('2 roles')).toBeInTheDocument();
  });

  it('renders no subtitle line at all when none is given', () => {
    const { container } = render(<MemberIdentity name='ravenne' />);
    // Name + avatar only: two inner spans under the flex-col wrapper, not three.
    expect(container.querySelectorAll('.flex-col > span')).toHaveLength(1);
  });
});
