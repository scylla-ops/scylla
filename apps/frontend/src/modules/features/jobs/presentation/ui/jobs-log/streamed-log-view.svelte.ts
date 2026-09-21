import { EditorView } from '@codemirror/view';

/** Distance from the end that still counts as "parked at the tail", in px. */
const TAIL_THRESHOLD_PX = 24;

export interface StreamedLogView {
  /** Hand to `renderCodeMirror`'s `onView`. */
  attach: (view: EditorView | null) => void;
}

/**
 * Feeds a growing log stream into a CodeMirror viewer, and keeps the viewport on
 * the tail the way an IDE console does: it opens on the end of the log, sticks
 * to it while lines arrive, and hands control back the moment the reader takes
 * over — scrolling up, or pressing on a line to select it. Following resumes by
 * itself once they are back at the end with nothing selected.
 *
 * **The document is written only here, by appending the delta.** Re-syncing a
 * changed value over the whole document — `changes: { from: 0, to: length }`,
 * which is what `@uiw/react-codemirror` did on every `value` change — drops the
 * scroll offset and collapses the selection. At one flush per 150 ms that reads
 * as "the log keeps jumping back to the top and I can't select anything". The
 * action next door takes `doc` once and never looks at it again for the same
 * reason.
 *
 * Appending assumes the stream only ever grows or restarts from scratch, which
 * is what `createTailJobLogs` produces (a cumulative `lines.join('\n')`).
 *
 * `logs` is a getter, read inside an `$effect`: that is the subscription. The
 * editor arrives through {@link StreamedLogView.attach} rather than being looked
 * up, because there is no moment before the action runs at which it exists.
 */
export const createStreamedLogView = (logs: () => string): StreamedLogView => {
  let view = $state<EditorView | null>(null);
  let isFollowing = true;
  let hasAnchored = false;

  // 1. The document: append whatever is new since the last write.
  $effect(() => {
    const text = logs();
    if (!view) return;

    const written = view.state.doc.length;
    if (text.length === written) return;

    // Shorter than what is on screen ⇒ the stream restarted (another job, or a
    // reconnect): nothing of the old document is worth keeping.
    view.dispatch({
      changes:
        text.length > written
          ? { from: written, insert: text.slice(written) }
          : { from: 0, to: written, insert: text },
    });
  });

  // 2. The viewport: follow the tail, unless the reader has taken over.
  $effect(() => {
    const text = logs();
    if (!view || !isFollowing || text.length === 0) return;

    const end = view.state.doc.length;
    if (end === 0) return;

    // Opening an already-finished job hands over the whole log at once.
    // Animating across it would scroll through lines CodeMirror has not
    // rendered yet — it only renders the viewport — and stop short of the end,
    // since the heights it scrolls against are estimates until measured. Land
    // on the end instead, and let `scrollIntoView` re-apply itself across the
    // measure passes.
    if (!hasAnchored) {
      hasAnchored = true;
      view.dispatch({ effects: EditorView.scrollIntoView(end, { y: 'end' }) });
      return;
    }

    // Afterwards the document only grows a few lines at a time: a frame late,
    // so they are laid out and `scrollHeight` is final, and smooth because the
    // jump is small enough to follow with the eye.
    const scroller = view.scrollDOM;
    const frame = requestAnimationFrame(() => {
      scroller.scrollTo({ top: scroller.scrollHeight, behavior: 'smooth' });
    });

    return () => cancelAnimationFrame(frame);
  });

  // 3. Who is in control: the reader, or the tail.
  $effect(() => {
    if (!view) return;

    const scroller = view.scrollDOM;
    const ownerDocument = scroller.ownerDocument;
    const editor = view;
    let frame = 0;

    const isAtTail = () =>
      scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight <= TAIL_THRESHOLD_PX;

    // Both selections are read: the viewer is not editable, so a drag leaves a
    // native browser selection, where an editable instance keeps it in CM state.
    const hasSelection = () => {
      if (!editor.state.selection.main.empty) return true;

      const selection = ownerDocument.getSelection();
      return !!selection && !selection.isCollapsed && scroller.contains(selection.anchorNode);
    };

    const resync = () => {
      isFollowing = isAtTail() && !hasSelection();
    };

    // Wheel and keys are handled before the box has actually moved, so the
    // position is only worth reading on the next frame.
    const handleUserScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(resync);
    };

    const handlePointerDown = () => {
      isFollowing = false;
      // A drag is not a scroll, so the browser keeps animating a smooth scroll
      // underneath it — that is what makes a selection slide away mid-drag.
      // Writing the current offset aborts it; a real user scroll aborts it alone.
      scroller.scrollTo({ top: scroller.scrollTop, behavior: 'instant' });
    };

    // Watched on the document: a drag started in the log can be released
    // outside it, and a touch scroll ends on `pointercancel` instead of
    // `pointerup`.
    const handleGestureEnd = () => resync();

    scroller.addEventListener('wheel', handleUserScroll, { passive: true });
    scroller.addEventListener('pointerdown', handlePointerDown);
    ownerDocument.addEventListener('keydown', handleUserScroll);
    ownerDocument.addEventListener('pointerup', handleGestureEnd);
    ownerDocument.addEventListener('pointercancel', handleGestureEnd);
    ownerDocument.addEventListener('touchend', handleGestureEnd);

    return () => {
      cancelAnimationFrame(frame);
      scroller.removeEventListener('wheel', handleUserScroll);
      scroller.removeEventListener('pointerdown', handlePointerDown);
      ownerDocument.removeEventListener('keydown', handleUserScroll);
      ownerDocument.removeEventListener('pointerup', handleGestureEnd);
      ownerDocument.removeEventListener('pointercancel', handleGestureEnd);
      ownerDocument.removeEventListener('touchend', handleGestureEnd);
    };
  });

  return {
    attach: (created: EditorView | null) => {
      view = created;
      if (!created) hasAnchored = false;
    },
  };
};
