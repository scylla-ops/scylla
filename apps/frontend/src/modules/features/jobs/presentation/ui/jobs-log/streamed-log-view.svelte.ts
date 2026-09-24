import { EditorView } from '@codemirror/view';

const TAIL_THRESHOLD_PX = 24;

export interface StreamedLogView {
  attach: (view: EditorView | null) => void;
}

/**
 * Feeds a growing log into CodeMirror and follows the tail like an IDE console,
 * until the reader scrolls up or selects; following resumes at the end.
 * The document is only appended to: replacing it would lose the scroll and the
 * selection. `logs` only grows, or restarts.
 */
export const createStreamedLogView = (logs: () => string): StreamedLogView => {
  let view = $state<EditorView | null>(null);
  let isFollowing = true;
  let hasAnchored = false;

  $effect(() => {
    const text = logs();
    if (!view) return;

    const written = view.state.doc.length;
    if (text.length === written) return;

    // Shorter than on screen: the stream restarted.
    view.dispatch({
      changes:
        text.length > written
          ? { from: written, insert: text.slice(written) }
          : { from: 0, to: written, insert: text },
    });
  });

  $effect(() => {
    const text = logs();
    if (!view || !isFollowing || text.length === 0) return;

    const end = view.state.doc.length;
    if (end === 0) return;

    // A finished job arrives whole: jump to the end (CodeMirror only renders the viewport).
    if (!hasAnchored) {
      hasAnchored = true;
      view.dispatch({ effects: EditorView.scrollIntoView(end, { y: 'end' }) });
      return;
    }

    // Then a few lines at a time: a frame later, smoothly.
    const scroller = view.scrollDOM;
    const frame = requestAnimationFrame(() => {
      scroller.scrollTo({ top: scroller.scrollHeight, behavior: 'smooth' });
    });

    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    if (!view) return;

    const scroller = view.scrollDOM;
    const ownerDocument = scroller.ownerDocument;
    const editor = view;
    let frame = 0;

    const isAtTail = () =>
      scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight <= TAIL_THRESHOLD_PX;

    // Read-only viewer: a drag leaves a native selection.
    const hasSelection = () => {
      if (!editor.state.selection.main.empty) return true;

      const selection = ownerDocument.getSelection();
      return !!selection && !selection.isCollapsed && scroller.contains(selection.anchorNode);
    };

    const resync = () => {
      isFollowing = isAtTail() && !hasSelection();
    };

    // Wheel and keys fire before the scroll: read the position a frame later.
    const handleUserScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(resync);
    };

    const handlePointerDown = () => {
      isFollowing = false;
      // Abort the smooth scroll under a drag, or the selection slides away.
      scroller.scrollTo({ top: scroller.scrollTop, behavior: 'instant' });
    };

    // On the document: a drag can end outside the log, and a touch scroll ends on `pointercancel`.
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
