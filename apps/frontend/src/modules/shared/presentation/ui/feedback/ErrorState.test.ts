import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import ErrorState from './ErrorState.svelte';

describe('ErrorState', () => {
  it('falls back to a generic title and message', () => {
    render(ErrorState);

    expect(screen.getByText('Error')).toBeInTheDocument();
    expect(screen.getByText('An error occurred')).toBeInTheDocument();
  });

  it('shows the caller title and message instead', () => {
    render(ErrorState, { title: 'Nothing loaded', message: 'Unable to load triggers' });

    expect(screen.getByText('Nothing loaded')).toBeInTheDocument();
    expect(screen.getByText('Unable to load triggers')).toBeInTheDocument();
    expect(screen.queryByText('An error occurred')).toBeNull();
  });
});
