import { useCallback, useEffect, useState } from 'react';

/**
 * The height, in pixels, of however much room the layout leaves the element
 * `containerRef` is put on — for the components that have to size themselves to
 * the space they were given rather than to their content.
 *
 * Attach it to a container whose height comes from the layout (`flex-1
 * min-h-0`, say) and never from what is inside it, or the measurement feeds
 * back into itself: children sized from the height grow the container, which
 * reports a new height, which resizes the children.
 *
 * `containerRef` is a callback ref, not a `useRef`: an element that mounts after
 * the first render (behind a loading state, or a panel that opens) never
 * re-runs an effect keyed on a ref object, so the observer would attach to
 * nothing and silently never retry. The height is read on attach rather than
 * from the observer's first, asynchronous callback, so it is already known in
 * the commit the container appears in and nothing renders at a placeholder size
 * first.
 */
export const useMeasuredHeight = () => {
  const [container, setContainer] = useState<HTMLElement | null>(null);
  const [height, setHeight] = useState<number | null>(null);

  useEffect(() => {
    if (!container) return;

    const observer = new ResizeObserver(([entry]) => setHeight(entry.contentRect.height));
    observer.observe(container);
    return () => observer.disconnect();
  }, [container]);

  const containerRef = useCallback((node: HTMLElement | null) => {
    setContainer(node);
    if (node) setHeight(node.clientHeight);
  }, []);

  return { height, containerRef };
};
