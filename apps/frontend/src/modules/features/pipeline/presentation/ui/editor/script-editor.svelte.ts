import { EditorView } from '@codemirror/view';
import type { Extension } from '@codemirror/state';

export interface ScriptEditorParams {
  value: () => string;
  onChange: (value: string) => void;
  extensions?: () => Extension[];
}

/**
 * Two-way binding between CodeMirror and a string. `renderCodeMirror` reads `doc`
 * once: a changed value is dispatched here, only when it did not come from the editor.
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
    attach: (next: EditorView | null) => {
      view = next;
    },
  };
};
