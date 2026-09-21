import type { Action } from 'svelte/action';

/** Below this width an action bar can no longer lay its buttons out inline. */
const COMPACT_WIDTH_PX = 70;

export interface CompactContainer {
  /** True once the observed element is narrower than the threshold. */
  readonly isCompact: boolean;
  /** `use:measure` on the element whose width decides the layout. */
  measure: Action<HTMLElement>;
}

/**
 * Reports when an element is too narrow to lay its content out inline, so a row
 * action bar can fall back to a dropdown — the Svelte counterpart of
 * `use-compact-container.ts`.
 *
 * The breakpoint is the element's **own** width, not the viewport's: table
 * columns are sized independently of it.
 *
 * An action, not an effect on a ref: the element it watches lives in a table
 * row and is created and destroyed with it, which is precisely the case a
 * `useRef` + effect pair kept getting wrong.
 */
export const createCompactContainer = (threshold: number = COMPACT_WIDTH_PX): CompactContainer => {
  let isCompact = $state(false);

  const measure: Action<HTMLElement> = node => {
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        isCompact = entry.contentRect.width < threshold;
      }
    });
    observer.observe(node);

    return {
      destroy() {
        observer.disconnect();
      },
    };
  };

  return {
    get isCompact() {
      return isCompact;
    },
    measure,
  };
};
