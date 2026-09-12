import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { StartNodeFormDialog } from './StartNodeFormDialog';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('StartNodeFormDialog', () => {
  it('prefills the name field with the current pipeline name', () => {
    renderWithI18n(
      <StartNodeFormDialog open currentName='my-pipeline' onSave={vi.fn()} onOpenChange={vi.fn()} />,
    );
    expect(screen.getByLabelText('Name')).toHaveValue('my-pipeline');
  });

  it('saving trims the name, calls onSave and closes', async () => {
    const onSave = vi.fn();
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <StartNodeFormDialog open currentName='my-pipeline' onSave={onSave} onOpenChange={onOpenChange} />,
    );

    const input = screen.getByLabelText('Name');
    await user.clear(input);
    await user.type(input, 'renamed');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSave).toHaveBeenCalledWith('renamed');
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('a name with whitespace fails the no-whitespace pattern and disables Save', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <StartNodeFormDialog open currentName='my-pipeline' onSave={vi.fn()} onOpenChange={vi.fn()} />,
    );

    const input = screen.getByLabelText('Name');
    await user.clear(input);
    await user.type(input, 'has space');

    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('an empty name disables Save entirely', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <StartNodeFormDialog open currentName='my-pipeline' onSave={vi.fn()} onOpenChange={vi.fn()} />,
    );

    await user.clear(screen.getByLabelText('Name'));
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });
});
