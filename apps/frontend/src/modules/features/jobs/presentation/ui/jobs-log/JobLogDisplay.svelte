<script lang="ts">
  import { EditorState } from '@codemirror/state';
  import { EditorView, lineNumbers } from '@codemirror/view';
  import { renderCodeMirror } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { createTailJobLogs } from '../../tail-job-logs.svelte.ts';
  import { createStreamedLogView } from './streamed-log-view.svelte.ts';
  import { jobsMessages } from '../jobs.messages.ts';

  /** Fallback for a caller that has no measured room to offer, in pixels. */
  const DEFAULT_MAX_HEIGHT = 448;

  interface Props {
    jobId: string;
    nodeId?: string;
    /** How tall the log may grow before it scrolls, in pixels. */
    maxHeight?: number;
  }

  let { jobId, nodeId, maxHeight }: Props = $props();

  const tail = createTailJobLogs({ jobId: () => jobId, nodeId: () => nodeId });
  const stream = createStreamedLogView(() => tail.text);

  /**
   * Explicitly, rather than the wrapper's `basicSetup`.
   *
   * That default brought history, autocompletion, linting and search along —
   * every one of them useless to a log nobody can type into, and all of them in
   * the initial `vendor-codemirror` chunk. Line numbers and a read-only state
   * are the whole of what this view is.
   *
   * Native selection is deliberate too: `drawSelection` is not here, so a drag
   * leaves a browser selection, which is what `createStreamedLogView` reads to
   * know the reader is selecting rather than idling at the tail.
   */
  const extensions = $derived([
    lineNumbers(),
    EditorState.readOnly.of(true),
    EditorView.editable.of(false),
    EditorView.theme({ '&': { maxHeight: `${maxHeight ?? DEFAULT_MAX_HEIGHT}px` } }),
  ]);
</script>

<!--
  A job's logs, whole or per node: both come from the same streaming source
  (full persisted history, then the live tail), so the view is complete and live
  regardless of when it is opened.

  It grows with the log up to `maxHeight` and scrolls past it, rather than
  standing at a fixed size: the room a log is worth is the room the page has
  left, which only the caller laying the panels out can know.
-->
{#if tail.isLoading}
  <div>{t(jobsMessages.loading)}</div>
{:else if tail.isError}
  <div>{t(jobsMessages.logsError)}</div>
{:else}
  <div class="min-w-0 w-full overflow-hidden">
    <div use:renderCodeMirror={{ extensions, onView: stream.attach }}></div>
  </div>
{/if}
