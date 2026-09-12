import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MembersHint } from './MembersHint';

describe('MembersHint', () => {
  it('renders its children next to an info icon', () => {
    const { container } = render(<MembersHint>Users seen through a project grant aren't shown here.</MembersHint>);
    expect(
      screen.getByText("Users seen through a project grant aren't shown here."),
    ).toBeInTheDocument();
    expect(container.querySelector('.lucide-info')).toBeInTheDocument();
  });
});
