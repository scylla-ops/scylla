import type { Action } from 'svelte/action';

export interface MeasuredHeight {
  /** `null` until the element the action is on has been attached. */
  readonly height: number | null;
  /** `use:measure` on the container whose height the layout decides. */
  measure: Action<HTMLElement>;
}

/**
 * The height, in pixels, of however much room the layout leaves an element —
 * for the components that size themselves to the space they were given rather
 * than to their content.
 *
 * Put `use:measure` on a container whose height comes from the layout (`flex-1
 * min-h-0`, say) and never from what is inside it, or the measurement feeds back
 * into itself: children sized from the height grow the container, which reports
 * a new height, which resizes the children.
 *
 * A Svelte action is the whole of what React needed a callback ref plus an
 * effect for. The ref had to be a *callback* ref because an element that mounts
 * later — behind a loading state, or in a panel that opens — never re-runs an
 * effect keyed on a ref object; an action has no such failure mode, since it
 * runs when its element is created, whenever that is.
 *
 * The height is read on attach rather than waiting for the observer's first,
 * asynchronous callback, so it is already known on the frame the container
 * appears and nothing renders at a placeholder size first.
 */
export const createMeasuredHeight = (): MeasuredHeight => {
  let height = $state<number | null>(null);

  const measure: Action<HTMLElement> = node => {
    height = node.clientHeight;

    const observer = new ResizeObserver(([entry]) => {
      height = entry.contentRect.height;
    });
    observer.observe(node);

    return {
      destroy() {
        observer.disconnect();
        height = null;
      },
    };
  };

  return {
    get height() {
      return height;
    },
    measure,
  };
};
