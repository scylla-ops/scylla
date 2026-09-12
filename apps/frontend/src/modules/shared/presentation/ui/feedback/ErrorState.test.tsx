import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { ErrorState } from './ErrorState';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

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
