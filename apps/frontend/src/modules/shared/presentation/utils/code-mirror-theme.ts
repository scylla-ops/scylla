import { EditorView, oneDarkHighlightStyle } from '@uiw/react-codemirror';
import type { Extension } from '@uiw/react-codemirror';
import { syntaxHighlighting } from '@codemirror/language';

export interface CodeMirrorThemeOptions {
  /** Current app color scheme — drives CodeMirror's own dark defaults + token colors. */
  isDark: boolean;
  /** Error mode turns the accent (resting border + focus bar) destructive. */
  hasError?: boolean;
}

/**
 * Builds the editor theme from the app's CSS variables so it follows light/dark
 * automatically. Passed to `<ReactCodeMirror theme={...} />` (not `extensions`)
 * so it replaces the library's hardcoded white default instead of fighting it.
 *
 * In error mode the accent (left bar on focus + the resting border) turns
 * destructive so it reads as a single coherent state instead of an outer red
 * ring fighting the inner focus shadow.
 */
export const buildCodeMirrorTheme = ({
  isDark,
  hasError = false,
}: CodeMirrorThemeOptions): Extension => {
  const accent = hasError ? 'var(--destructive)' : 'var(--primary)';

  const theme = EditorView.theme(
    {
      '&': {
        backgroundColor: 'var(--code-editor-bg)',
        color: 'var(--foreground)',
        borderRadius: '0.75rem',
        overflow: 'hidden',
        maxWidth: '100%',
        border: `1px solid ${hasError ? 'var(--destructive)' : 'var(--border)'}`,
        transition: 'border-color 0.1s ease-in-out, box-shadow 0.1s ease-in-out',
      },
      '.cm-scroller': { overflow: 'auto', backgroundColor: 'var(--code-editor-bg)' },
      '.cm-content': { padding: '0.5rem', caretColor: 'var(--foreground)' },
      '.cm-placeholder': { color: 'var(--muted-foreground)' },
      '.cm-activeLine': { backgroundColor: 'var(--code-editor-line-bg)' },
      '.cm-activeLineGutter': {
        backgroundColor: 'var(--code-editor-line-bg)',
        color: 'var(--foreground)',
      },
      '.cm-lineNumbers': { color: 'var(--muted-foreground)' },
      '.cm-gutters': {
        backgroundColor: 'var(--code-editor-bg)',
        borderRight: '1px solid var(--border)',
        color: 'var(--muted-foreground)',
      },
      '.cm-foldPlaceholder': {
        backgroundColor: 'var(--muted)',
        color: 'var(--muted-foreground)',
        border: 'none',
      },
      '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--foreground)' },
      '.cm-selectionBackground, &.cm-focused .cm-selectionBackground, .cm-content ::selection': {
        backgroundColor: 'var(--code-editor-selection-bg)',
      },
      '.cm-selectionMatch': { backgroundColor: 'var(--code-editor-selection-bg)' },
      '.cm-matchingBracket, .cm-nonmatchingBracket': {
        backgroundColor: 'var(--code-editor-selection-bg)',
        outline: '1px solid var(--border)',
      },
      '.cm-panels, .cm-tooltip': {
        backgroundColor: 'var(--popover)',
        color: 'var(--popover-foreground)',
        border: '1px solid var(--border)',
      },
      '.cm-tooltip-autocomplete ul li[aria-selected]': {
        backgroundColor: 'var(--accent)',
        color: 'var(--accent-foreground)',
      },
      '&.cm-focused': {
        outline: 'none',
        boxShadow: `-2px 0 0px 0px ${accent}, 0 1px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06)`,
        borderRadius: '0.75rem',
      },
    },
    { dark: isDark },
  );

  // The library's default token palette is tuned for light backgrounds (dark
  // red strings, purple keywords) and is unreadable on the dark surface.
  return isDark ? [theme, syntaxHighlighting(oneDarkHighlightStyle)] : theme;
};
