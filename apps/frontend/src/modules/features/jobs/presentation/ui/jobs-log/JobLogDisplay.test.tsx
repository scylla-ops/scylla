import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
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
    default: ({ value, maxHeight }: { value: string; maxHeight?: string }) => (
      <div data-testid='log-editor' data-max-height={maxHeight}>
        {value}
      </div>
    ),
  };
});

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

  it('scrolls only past the height the caller measured for it', () => {
    renderWithI18n(<JobLogDisplay jobId='job-1' maxHeight={720} />);
    expect(screen.getByTestId('log-editor')).toHaveAttribute('data-max-height', '720px');
  });

  it('falls back to a fixed height when the caller measured none', () => {
    renderWithI18n(<JobLogDisplay jobId='job-1' />);
    expect(screen.getByTestId('log-editor')).toHaveAttribute('data-max-height', '448px');
  });
});
