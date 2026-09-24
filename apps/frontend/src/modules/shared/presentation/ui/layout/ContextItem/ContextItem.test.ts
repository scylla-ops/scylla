import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import Building2 from '@lucide/svelte/icons/building-2';
import { render } from '@/test/render.svelte.ts';
import ContextItem from './ContextItem.svelte';

describe('ContextItem', () => {
  it('shows the name', () => {
    render(ContextItem, { name: 'Acme Corp', icon: Building2 });

    expect(screen.getByText('Acme Corp')).toBeInTheDocument();
  });

  it('shows the description when given', () => {
    render(ContextItem, { name: 'Acme Corp', description: '12 projects', icon: Building2 });

    expect(screen.getByText('12 projects')).toBeInTheDocument();
  });

  it('omits the description entirely when not given', () => {
    const { container } = render(ContextItem, { name: 'Acme Corp', icon: Building2 });

    expect(container.querySelectorAll('span.block')).toHaveLength(1);
  });
});
