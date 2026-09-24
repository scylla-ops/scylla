import { EditorView } from '@codemirror/view';
import { oneDarkHighlightStyle } from '@codemirror/theme-one-dark';
import type { Extension } from '@codemirror/state';
import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language';

export interface CodeMirrorThemeOptions {
  isDark: boolean;
  /** Turns the accent (border, focus bar) destructive. */
  hasError?: boolean;
}

/** The editor theme, from the app's CSS variables, so it follows light/dark. */
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
      // Must stay translucent: the selection layer is drawn under the lines.
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
      // The full path is needed to beat the base theme's `.cm-selectionBackground` on specificity.
      '.cm-selectionBackground, .cm-content ::selection, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground':
        {
          backgroundColor: 'var(--code-editor-selection-bg)',
        },
      '.cm-selectionMatch': { backgroundColor: 'var(--code-editor-match-bg)' },
      '.cm-matchingBracket, .cm-nonmatchingBracket': {
        backgroundColor: 'var(--code-editor-match-bg)',
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

  return [theme, syntaxHighlighting(isDark ? oneDarkHighlightStyle : defaultHighlightStyle)];
};
