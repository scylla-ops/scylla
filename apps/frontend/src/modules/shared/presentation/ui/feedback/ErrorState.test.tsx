import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { ErrorState } from './ErrorState';

describe('ErrorState', () => {
  it('shows the default title and message', () => {
    renderWithI18n(<ErrorState />);
    expect(screen.getByText('Error')).toBeInTheDocument();
    expect(screen.getByText('An error occurred')).toBeInTheDocument();
  });

  it('shows a custom title and message instead', () => {
    renderWithI18n(<ErrorState title='Pipeline not found' message='It may have been deleted.' />);
    expect(screen.getByText('Pipeline not found')).toBeInTheDocument();
    expect(screen.getByText('It may have been deleted.')).toBeInTheDocument();
    expect(screen.queryByText('An error occurred')).not.toBeInTheDocument();
  });
});
