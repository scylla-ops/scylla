<script lang="ts">
  import { EditorState } from '@codemirror/state';
  import { EditorView, lineNumbers } from '@codemirror/view';
  import { renderCodeMirror } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { createTailJobLogs } from '../../../tail-job-logs.svelte.ts';
  import { createStreamedLogView } from '../streamed-log-view.svelte.ts';
  import { jobsMessages } from '../../jobs.messages.ts';

  const DEFAULT_MAX_HEIGHT = 448;

  interface Props {
    jobId: string;
    nodeId?: string;
    maxHeight?: number;
  }

  let { jobId, nodeId, maxHeight }: Props = $props();

  const tail = createTailJobLogs({ jobId: () => jobId, nodeId: () => nodeId });
  const stream = createStreamedLogView(() => tail.text);

  /**
   * Only line numbers and read-only. No `drawSelection`: a drag leaves a native
   * selection, which `createStreamedLogView` reads.
   */
  const extensions = $derived([
    lineNumbers(),
    EditorState.readOnly.of(true),
    EditorView.editable.of(false),
    EditorView.theme({ '&': { maxHeight: `${maxHeight ?? DEFAULT_MAX_HEIGHT}px` } }),
  ]);
</script>

<!-- Grows with the log up to `maxHeight`, then scrolls: only the caller knows the room left. -->
{#if tail.isLoading}
  <div>{t(jobsMessages.loading)}</div>
{:else if tail.isError}
  <div>{t(jobsMessages.logsError)}</div>
{:else}
  <div class="min-w-0 w-full overflow-hidden">
    <div use:renderCodeMirror={{ extensions, onView: stream.attach }}></div>
  </div>
{/if}
