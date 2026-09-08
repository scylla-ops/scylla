import { useEffect, useRef, useState } from 'react';

/** Below this width an action bar can no longer lay its buttons out inline. */
const COMPACT_WIDTH_PX = 70;

/**
 * Reports when the observed element is too narrow to lay its content out inline,
 * so a row action bar can fall back to a dropdown. The breakpoint is the element's
 * own width, not the viewport's: table columns are sized independently of it.
 */
export const useCompactContainer = (threshold: number = COMPACT_WIDTH_PX) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isCompact, setIsCompact] = useState(false);

  useEffect(() => {
    const element = containerRef.current;
    if (!element) return;

    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        setIsCompact(entry.contentRect.width < threshold);
      }
    });
    observer.observe(element);

    return () => observer.disconnect();
  }, [threshold]);

  return { containerRef, isCompact };
};
