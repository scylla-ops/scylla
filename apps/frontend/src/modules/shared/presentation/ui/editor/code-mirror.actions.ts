import { Compartment, EditorState, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import type { Action } from 'svelte/action';
import { getTheme, subscribeToTheme } from '@shared/presentation/stores/theme.store.ts';
import { buildCodeMirrorTheme } from '@shared/presentation/utils/code-mirror-theme.ts';

export interface CodeMirrorOptions {
  /**
   * Read once, when the view is created. Replacing the whole document would lose
   * the scroll and the selection: the owner dispatches its changes through `onView`.
   */
  doc?: string;
  /** Reconfigured in place when it changes: the document stays. */
  extensions?: Extension[];
  hasError?: boolean;
  /** The live view once it exists, `null` after teardown. */
  onView?: (view: EditorView | null) => void;
}

/**
 * Mounts a CodeMirror 6 editor with the app's theme. Only the extensions the
 * caller gives are loaded. The theme is a `Compartment`: a light/dark switch
 * reconfigures it without rebuilding the view.
 */
export const renderCodeMirror: Action<HTMLElement, CodeMirrorOptions> = (node, options = {}) => {
  const themeCompartment = new Compartment();
  const extensionsCompartment = new Compartment();

  let hasError = options.hasError ?? false;
  let extensions = options.extensions ?? [];

  const theme = () => buildCodeMirrorTheme({ isDark: getTheme() !== 'light', hasError });

  const view = new EditorView({
    parent: node,
    state: EditorState.create({
      doc: options.doc ?? '',
      extensions: [themeCompartment.of(theme()), extensionsCompartment.of(extensions)],
    }),
  });

  const reconfigureTheme = () => view.dispatch({ effects: themeCompartment.reconfigure(theme()) });

  const unsubscribe = subscribeToTheme(reconfigureTheme);

  options.onView?.(view);

  return {
    update(next: CodeMirrorOptions) {
      if ((next.hasError ?? false) !== hasError) {
        hasError = next.hasError ?? false;
        reconfigureTheme();
      }

      // Compared by identity: a `$derived` gives a new array only when something changed.
      const nextExtensions = next.extensions ?? [];
      if (nextExtensions !== extensions) {
        extensions = nextExtensions;
        view.dispatch({ effects: extensionsCompartment.reconfigure(extensions) });
      }
    },
    destroy() {
      unsubscribe();
      options.onView?.(null);
      view.destroy();
    },
  };
};
