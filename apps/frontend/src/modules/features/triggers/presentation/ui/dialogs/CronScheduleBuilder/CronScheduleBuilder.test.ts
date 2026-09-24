import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import CronScheduleBuilder from './CronScheduleBuilder.svelte';

const chooseFrequency = (name: string) =>
  userEvent.click(screen.getByRole('radio', { name: new RegExp(name, 'i') }));

const pickFromSelect = async (label: string, option: string) => {
  await userEvent.click(screen.getByRole('combobox', { name: label }));
  await userEvent.click(await findFloating('option', option));
};

describe('CronScheduleBuilder', () => {
  it('does not emit before the user has touched anything', () => {
    // No emit on mount: the parent already has the initial value.
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 9 * * *', onChange });

    expect(onChange).not.toHaveBeenCalled();
  });

  it('opens on the frequency its initial expression describes', () => {
    render(CronScheduleBuilder, { initialValue: '0 9 * * *', onChange: vi.fn() });

    expect(screen.getByRole('radio', { name: /daily/i })).toBeChecked();
  });

  it('falls back to custom for an expression the builder cannot describe', () => {
    render(CronScheduleBuilder, { initialValue: '*/15 1-5 * * *', onChange: vi.fn() });

    expect(screen.getByRole('radio', { name: /custom/i })).toBeChecked();
    expect(screen.getByLabelText('Custom')).toHaveValue('*/15 1-5 * * *');
  });

  it('shows the resulting expression live', () => {
    render(CronScheduleBuilder, { initialValue: '30 6 * * *', onChange: vi.fn() });

    expect(screen.getByText('30 6 * * *')).toBeInTheDocument();
  });

  it('emits an hourly expression when the frequency changes', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 9 * * *', onChange });

    await chooseFrequency('hourly');

    expect(onChange).toHaveBeenLastCalledWith('0 * * * *');
  });

  it('emits the chosen minute for an hourly schedule', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 * * * *', onChange });

    await pickFromSelect('Minute', '15');

    expect(onChange).toHaveBeenLastCalledWith('15 * * * *');
  });

  it('emits minute and hour for a daily schedule', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 9 * * *', onChange });

    await pickFromSelect('Hour', '07');

    expect(onChange).toHaveBeenLastCalledWith('0 7 * * *');
  });

  it('emits the day of month for a monthly schedule', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 9 1 * *', onChange });

    expect(screen.getByRole('radio', { name: /monthly/i })).toBeChecked();
    await pickFromSelect('Day of the month', '12');

    expect(onChange).toHaveBeenLastCalledWith('0 9 12 * *');
  });

  it('emits the selected weekdays in ascending order', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '0 9 * * 5', onChange });

    expect(screen.getByRole('radio', { name: /weekly/i })).toBeChecked();

    // The days are sorted: the expression does not depend on the click order.
    const monday = screen.getAllByRole('button', { pressed: false })[0];
    await userEvent.click(monday);

    expect(onChange).toHaveBeenLastCalledWith(expect.stringMatching(/^0 9 \* \* \d(,\d)*$/));
    const [emitted] = onChange.mock.lastCall as [string];
    const days = emitted.split(' ')[4].split(',').map(Number);
    expect([...days].sort((a, b) => a - b)).toEqual(days);
  });

  it('emits exactly what was typed in custom mode', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '', onChange });

    await chooseFrequency('custom');
    const field = screen.getByLabelText('Custom');
    await userEvent.clear(field);
    await userEvent.type(field, '*/5 * * * *');

    expect(onChange).toHaveBeenLastCalledWith('*/5 * * * *');
  });

  it('seeds custom mode from the schedule that was built, not an empty box', async () => {
    const onChange = vi.fn();
    render(CronScheduleBuilder, { initialValue: '30 6 * * *', onChange });

    await chooseFrequency('custom');

    expect(screen.getByLabelText('Custom')).toHaveValue('30 6 * * *');
    expect(onChange).toHaveBeenLastCalledWith('30 6 * * *');
  });
});
