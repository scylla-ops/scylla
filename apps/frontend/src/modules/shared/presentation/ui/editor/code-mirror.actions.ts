import { Compartment, EditorState, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import type { Action } from 'svelte/action';
import { getTheme, subscribeToTheme } from '@shared/presentation/stores/theme.store.ts';
import { buildCodeMirrorTheme } from '@shared/presentation/utils/code-mirror-theme.ts';

export interface CodeMirrorOptions {
  /**
   * The document to open with.
   *
   * Read **once**, when the view is created, and never again. Writing a changed
   * `doc` back over the whole document is what `@uiw/react-codemirror` did on
   * every `value` change — `changes: { from: 0, to: doc.length }` — which drops
   * the scroll offset and collapses the selection. Whoever owns the content
   * dispatches its own changes through {@link CodeMirrorOptions.onView}.
   */
  doc?: string;
  /**
   * Extensions on top of the app theme: a language, line numbers, read-only…
   *
   * Reconfigured in place when it changes, so a caller may derive them from its
   * props. Rebuilding the view instead would throw the document away, which for
   * a log viewer means re-streaming it.
   */
  extensions?: Extension[];
  /** Renders the editor in destructive colours — an invalid script, say. */
  hasError?: boolean;
  /**
   * Receives the live view once it exists, and `null` when it is torn down.
   *
   * The counterpart of `onCreateEditor`, and for the same reason: everything
   * that drives the document from outside needs the instance, and there is no
   * moment before this one at which it exists.
   */
  onView?: (view: EditorView | null) => void;
}

/**
 * Mounts a CodeMirror 6 editor on an element, themed with the app's colours.
 *
 * This is the whole of what `@uiw/react-codemirror` was here for: creating the
 * view, tearing it down, and keeping its theme in step with light/dark. An
 * action is the exact shape of that job — it runs when its element is created,
 * whenever that is, and its `destroy` is the teardown — which is why the wrapper
 * leaves with React and CodeMirror itself does not.
 *
 * Nothing is configured beyond what the caller asks for. The wrapper defaulted
 * to `basicSetup` — history, autocompletion, linting, search — which a read-only
 * log viewer has no use for; extensions are now an explicit list per call site.
 *
 * The theme lives in a {@link Compartment} so a colour-scheme change
 * reconfigures it in place. Rebuilding the view instead would throw the
 * document, the scroll offset and the selection away every time the theme
 * flipped.
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

      // Identity, not contents: a caller derives this list, so a `$derived`
      // hands over a new array only when something it reads actually changed.
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
