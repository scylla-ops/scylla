import { EditorView } from '@uiw/react-codemirror';

/**
 * Builds the editor theme. In error mode the accent (left bar on focus + a
 * resting border) turns destructive so it reads as a single coherent state
 * instead of an outer red ring fighting the inner focus shadow.
 */
const buildTheme = (hasError: boolean) => {
  const accent = hasError ? 'var(--destructive)' : 'var(--primary)';

  return EditorView.theme({
    '&': {
      borderRadius: '0.75rem',
      overflow: 'hidden',
      maxWidth: '100%',
      border: `1px solid ${hasError ? 'var(--destructive)' : 'transparent'}`,
      transition: 'border-color 0.1s ease-in-out, box-shadow 0.1s ease-in-out',
    },
    '.cm-scroller': { overflow: 'auto' },
    '.cm-content': { padding: '0.5rem' },
    '.cm-activeLineGutter': { backgroundColor: 'var(--code-editor-line-bg)' },
    '.cm-activeLine': { backgroundColor: 'var(--code-editor-line-bg)' },
    'cm-gutters': { backgroundColor: 'transparent', border: 'none' },
    '&.cm-focused': {
      outline: 'none',
      boxShadow: `-2px 0 0px 0px ${accent}, 0 1px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06)`,
      borderRadius: '0.75rem',
    },
  });
};

export const codeMirrorTheme = buildTheme(false);
export const codeMirrorErrorTheme = buildTheme(true);
