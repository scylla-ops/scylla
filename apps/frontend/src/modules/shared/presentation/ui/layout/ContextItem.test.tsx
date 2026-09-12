import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ContextItem } from './ContextItem';
import { Building2 } from 'lucide-react';

describe('ContextItem', () => {
  it('shows the name', () => {
    render(<ContextItem name='Acme Corp' icon={Building2} />);
    expect(screen.getByText('Acme Corp')).toBeInTheDocument();
  });

  it('shows the description when given', () => {
    render(<ContextItem name='Acme Corp' description='12 projects' icon={Building2} />);
    expect(screen.getByText('12 projects')).toBeInTheDocument();
  });

  it('omits the description entirely when not given', () => {
    const { container } = render(<ContextItem name='Acme Corp' icon={Building2} />);
    // Only one text span (the name) - no second span for a missing description.
    expect(container.querySelectorAll('span.block')).toHaveLength(1);
  });
});
