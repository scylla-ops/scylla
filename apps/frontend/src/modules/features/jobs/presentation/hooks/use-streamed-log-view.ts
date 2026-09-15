import { useCallback, useEffect, useRef, useState } from 'react';
import { EditorView } from '@uiw/react-codemirror';

/** Distance from the end that still counts as "parked at the tail", in px. */
const TAIL_THRESHOLD_PX = 24;

/**
 * Feeds a growing log stream into a CodeMirror viewer, and keeps the viewport on
 * the tail the way an IDE console does: it opens on the end of the log, sticks to
 * it while lines arrive, and hands control back the moment the user takes over —
 * scrolling up, or pressing on a line to select it. Following resumes by itself
 * once the user is back at the end with nothing selected.
 *
 * The document is written **only here**, by appending the delta. `<ReactCodeMirror
 * value={...}>` re-syncs a changed `value` with `changes: { from: 0, to: length }`
 * — a whole-document replacement — which drops the scroll offset and collapses
 * the selection. At one flush per 150ms that reads as "the log keeps jumping back
 * to the top and I can't select anything". Hence the frozen `initialValue`: the
 * prop must never change again, or the library takes the document back over.
 *
 * Appending assumes the stream only ever grows or restarts from scratch, which is
 * what `useTailJobLogs` produces (a cumulative `lines.join('\n')`).
 *
 * The editor comes from `onCreateEditor` rather than a ref: `ReactCodeMirrorRef`
 * is populated one commit *after* mount, so an effect reading it would attach its
 * listeners to nothing and silently never retry.
 */
export const useStreamedLogView = (logs: string) => {
  const [view, setView] = useState<EditorView | null>(null);
  const [initialValue] = useState(() => logs);
  const isFollowingRef = useRef(true);
  const hasAnchoredRef = useRef(false);

  useEffect(() => {
    if (!view) return;

    const written = view.state.doc.length;
    if (logs.length === written) return;

    // Shorter than what is on screen ⇒ the stream restarted (another job, or a
    // reconnect): nothing of the old document is worth keeping.
    const changes =
      logs.length > written
        ? { from: written, insert: logs.slice(written) }
        : { from: 0, to: written, insert: logs };

    view.dispatch({ changes });
  }, [logs, view]);

  useEffect(() => {
    if (!view || !isFollowingRef.current) return;

    const end = view.state.doc.length;
    if (end === 0) return;

    // Opening an already-finished job hands over the whole log at once. Animating
    // across it would scroll through lines CodeMirror has not rendered yet (it only
    // renders the viewport) and stop short of the end, since the heights it scrolls
    // against are estimates until measured. Land on the end instead, and let
    // `scrollIntoView` re-apply itself across the measure passes.
    if (!hasAnchoredRef.current) {
      hasAnchoredRef.current = true;
      view.dispatch({ effects: EditorView.scrollIntoView(end, { y: 'end' }) });
      return;
    }

    // Afterwards the document only grows a few lines at a time: a frame late, so
    // they are laid out and `scrollHeight` is final, and smooth because the jump
    // is small enough to follow with the eye.
    const frame = requestAnimationFrame(() => {
      view.scrollDOM.scrollTo({ top: view.scrollDOM.scrollHeight, behavior: 'smooth' });
    });

    return () => cancelAnimationFrame(frame);
  }, [logs, view]);

  useEffect(() => {
    if (!view) return;

    const scroller = view.scrollDOM;
    const ownerDocument = scroller.ownerDocument;
    let frame = 0;

    const isAtTail = () =>
      scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight <= TAIL_THRESHOLD_PX;

    // Both selections are read: the viewer is `editable={false}`, so a drag leaves
    // a native browser selection, where an editable instance keeps it in CM state.
    const hasSelection = () => {
      if (!view.state.selection.main.empty) return true;

      const selection = ownerDocument.getSelection();
      return !!selection && !selection.isCollapsed && scroller.contains(selection.anchorNode);
    };

    const resync = () => {
      isFollowingRef.current = isAtTail() && !hasSelection();
    };

    // Wheel and keys are handled before the box has actually moved, so the
    // position is only worth reading on the next frame.
    const handleUserScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(resync);
    };

    const handlePointerDown = () => {
      isFollowingRef.current = false;
      // A drag is not a scroll, so the browser keeps animating a smooth scroll
      // underneath it — that is what makes a selection slide away mid-drag.
      // Writing the current offset aborts it; a real user scroll aborts it alone.
      scroller.scrollTo({ top: scroller.scrollTop, behavior: 'instant' });
    };

    // Watched on the document: a drag started in the log can be released outside
    // it, and a touch scroll ends on `pointercancel` instead of `pointerup`.
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
  }, [view]);

  return {
    /** Initial document only — see above, this must stay stable across renders. */
    initialValue,
    onCreateEditor: useCallback((created: EditorView) => setView(created), []),
  };
};
