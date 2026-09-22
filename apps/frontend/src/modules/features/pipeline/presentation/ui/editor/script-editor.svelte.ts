import { EditorView } from '@codemirror/view';
import type { Extension } from '@codemirror/state';

export interface ScriptEditorParams {
  /** The document, as the component owns it. */
  value: () => string;
  /** Called for every edit made inside the editor. */
  onChange: (value: string) => void;
  /** A language mode, line numbers… — whatever this editor is for. */
  extensions?: () => Extension[];
}

/**
 * Two-way binding between a CodeMirror document and a string owned by Svelte.
 *
 * `renderCodeMirror` deliberately reads its `doc` once and never again — that
 * is the bug it was written to avoid, because rewriting the whole document on
 * every keystroke is what `@uiw/react-codemirror` did, and it dropped the
 * scroll offset and collapsed the selection each time. So pushing a *changed*
 * document in is the caller's job, and it is only ever right to do when the
 * change did not come from the editor itself.
 *
 * Which is what the guard below is: compare before dispatching. The blueprint
 * tab rewriting the steps must reach the text editor; the text editor's own
 * keystroke coming back round must not.
 */
export const createScriptEditor = (params: ScriptEditorParams) => {
  let view = $state<EditorView | null>(null);

  const extensions = $derived([
    ...(params.extensions?.() ?? []),
    EditorView.updateListener.of(update => {
      if (update.docChanged) params.onChange(update.state.doc.toString());
    }),
  ]);

  $effect(() => {
    const next = params.value();
    if (!view) return;

    const current = view.state.doc.toString();
    if (current === next) return;

    view.dispatch({ changes: { from: 0, to: current.length, insert: next } });
  });

  return {
    get extensions() {
      return extensions;
    },
    /** Hand this to `renderCodeMirror`'s `onView`. */
    attach: (next: EditorView | null) => {
      view = next;
    },
  };
};
