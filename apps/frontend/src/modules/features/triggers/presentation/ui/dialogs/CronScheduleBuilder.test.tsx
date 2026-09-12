import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { CronScheduleBuilder } from './CronScheduleBuilder';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

const chooseFrequency = async (user: ReturnType<typeof userEvent.setup>, label: string) => {
  await user.click(screen.getByText(label));
};

const pickOption = async (
  user: ReturnType<typeof userEvent.setup>,
  combobox: HTMLElement,
  optionText: string,
) => {
  await user.click(combobox);
  await user.click(await screen.findByText(optionText));
};

describe('CronScheduleBuilder', () => {
  it('parses the initial value once on mount and re-emits the same cron', () => {
    const onChange = vi.fn();
    renderWithI18n(<CronScheduleBuilder initialValue='30 9 * * *' onChange={onChange} />);
    expect(onChange).toHaveBeenCalledWith('30 9 * * *');
    expect(onChange).toHaveBeenCalledTimes(1);
  });

  it('an unrecognized expression falls back to custom mode, showing it verbatim', () => {
    const onChange = vi.fn();
    renderWithI18n(<CronScheduleBuilder initialValue='*/15 * * * *' onChange={onChange} />);
    expect(screen.getByDisplayValue('*/15 * * * *')).toBeInTheDocument();
    expect(onChange).toHaveBeenCalledWith('*/15 * * * *');
  });

  it('switching frequency shows only that frequency\'s controls', async () => {
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 * * *' onChange={vi.fn()} />);

    // Starts on 'daily': the TimePicker's "at" + "local time" wording is visible.
    expect(screen.getByText('local time')).toBeInTheDocument();

    await chooseFrequency(user, 'Hourly');
    expect(screen.getByText('at minute')).toBeInTheDocument();
    expect(screen.queryByText('local time')).not.toBeInTheDocument();

    await chooseFrequency(user, 'Weekly');
    expect(screen.getByText('Mon')).toBeInTheDocument();
    expect(screen.getByText('local time')).toBeInTheDocument();

    await chooseFrequency(user, 'Monthly');
    expect(screen.getByText('on day')).toBeInTheDocument();

    await chooseFrequency(user, 'Custom');
    expect(screen.getByPlaceholderText('*/15 * * * *')).toBeInTheDocument();
  });

  it('hourly: picking a minute emits "<m> * * * *"', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 * * * *' onChange={onChange} />);
    await chooseFrequency(user, 'Hourly');

    await pickOption(user, screen.getByRole('combobox'), '45');
    expect(onChange).toHaveBeenLastCalledWith('45 * * * *');
  });

  it('daily: changing the hour and minute emits "<m> <h> * * *"', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 * * *' onChange={onChange} />);

    const [hourBox, minuteBox] = screen.getAllByRole('combobox');
    await pickOption(user, hourBox, '14');
    expect(onChange).toHaveBeenLastCalledWith('0 14 * * *');

    await pickOption(user, minuteBox, '30');
    expect(onChange).toHaveBeenLastCalledWith('30 14 * * *');
  });

  it('weekly: toggling weekdays sorts them into the dow field, and none selected renders "*"', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 * * 1,2,3,4,5' onChange={onChange} />);

    // Deselect every weekday one at a time.
    for (const day of ['Mon', 'Tue', 'Wed', 'Thu', 'Fri']) {
      await user.click(screen.getByText(day));
    }
    expect(onChange).toHaveBeenLastCalledWith('0 9 * * *');

    await user.click(screen.getByText('Sat'));
    await user.click(screen.getByText('Mon'));
    expect(onChange).toHaveBeenLastCalledWith('0 9 * * 1,6');
  });

  it('weekly: changing the time updates the schedule while keeping the chosen days', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 * * 1' onChange={onChange} />);

    const [hourBox] = screen.getAllByRole('combobox');
    await pickOption(user, hourBox, '18');
    expect(onChange).toHaveBeenLastCalledWith('0 18 * * 1');
  });

  it('monthly: changing the day of month emits "<m> <h> <dom> * *"', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 1 * *' onChange={onChange} />);

    const [dayBox] = screen.getAllByRole('combobox');
    await pickOption(user, dayBox, '15');
    expect(onChange).toHaveBeenLastCalledWith('0 9 15 * *');
  });

  it('custom: prefills with the previously built cron when switching in from another frequency', async () => {
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='30 9 * * *' onChange={vi.fn()} />);

    await chooseFrequency(user, 'Custom');
    expect(screen.getByDisplayValue('30 9 * * *')).toBeInTheDocument();
  });

  it('custom: typing an expression emits it trimmed, verbatim', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CronScheduleBuilder initialValue='*/10 * * * *' onChange={onChange} />);

    const input = screen.getByDisplayValue('*/10 * * * *');
    await user.clear(input);
    await user.type(input, '0 0 1 1 *');
    expect(onChange).toHaveBeenLastCalledWith('0 0 1 1 *');
  });

  it('shows the live plain-language summary and the rendered expression', () => {
    renderWithI18n(<CronScheduleBuilder initialValue='0 9 * * *' onChange={vi.fn()} />);
    expect(screen.getByText('Every day at 09:00')).toBeInTheDocument();
    expect(screen.getByText('0 9 * * *')).toBeInTheDocument();
  });
});
