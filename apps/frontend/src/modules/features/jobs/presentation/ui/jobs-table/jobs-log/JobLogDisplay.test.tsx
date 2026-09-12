import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { JobLogDisplay } from './JobLogDisplay';
import type * as ReactCodeMirrorModule from '@uiw/react-codemirror';

const tailState: { logString: string; isLoading: boolean; isError: boolean } = {
  logString: '',
  isLoading: false,
  isError: false,
};
vi.mock('@/modules/features/jobs/presentation/hooks/use-tail-job-logs.ts', () => ({
  useTailJobLogs: () => tailState,
}));

vi.mock('@/modules/features/jobs/presentation/hooks/use-streamed-log-view.ts', () => ({
  useStreamedLogView: (logs: string) => ({ initialValue: logs, onCreateEditor: vi.fn() }),
}));

// Same rationale as StepNodeFormDialog's batch: the real editor needs DOM APIs
// jsdom doesn't have, but useCodeMirrorTheme (called by this same component)
// imports EditorView from this same package and must keep the real one.
vi.mock('@uiw/react-codemirror', async importOriginal => {
  const actual = await importOriginal<typeof ReactCodeMirrorModule>();
  return {
    ...actual,
    default: ({ value }: { value: string }) => <div data-testid='log-editor'>{value}</div>,
  };
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  tailState.logString = '';
  tailState.isLoading = false;
  tailState.isError = false;
});

describe('JobLogDisplay', () => {
  it('shows a loading message while the stream has not resolved yet', () => {
    tailState.isLoading = true;
    renderWithI18n(<JobLogDisplay jobId='job-1' />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(screen.queryByTestId('log-editor')).not.toBeInTheDocument();
  });

  it('shows an error message when the stream failed', () => {
    tailState.isError = true;
    renderWithI18n(<JobLogDisplay jobId='job-1' />);
    expect(screen.getByText('Error loading logs...')).toBeInTheDocument();
  });

  it('renders the accumulated log text once the stream has data', () => {
    tailState.logString = 'line one\nline two';
    renderWithI18n(<JobLogDisplay jobId='job-1' nodeId='node-1' />);
    expect(screen.getByTestId('log-editor')).toHaveTextContent('line one line two');
  });
});
