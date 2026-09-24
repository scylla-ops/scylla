// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { defaultHighlightStyle, highlightingFor } from '@codemirror/language';
import { buildCodeMirrorTheme } from './code-mirror-theme';

const highlightedTokens = [defaultHighlightStyle.specs[0].tag].flat();

const tokenClass = (isDark: boolean) =>
  highlightingFor(
    EditorState.create({ extensions: buildCodeMirrorTheme({ isDark }) }),
    highlightedTokens,
  );

describe('buildCodeMirrorTheme', () => {
  // CodeMirror's `Extension` is itself array-shaped internally (a Facet
  // provider), so `Array.isArray`/length can't distinguish "just the theme"
  // from "theme + syntaxHighlighting" - assert the one thing that must hold
  // instead: dark mode's extra oneDarkHighlightStyle makes it a genuinely
  // different extension from the light-mode one.
  it('dark mode adds the oneDarkHighlightStyle override, producing a different extension than light mode', () => {
    const light = buildCodeMirrorTheme({ isDark: false });
    const dark = buildCodeMirrorTheme({ isDark: true });
    expect(dark).not.toEqual(light);
  });

  it('colours the tokens in light mode, as it does in dark mode', () => {
    expect(tokenClass(false)).not.toBeNull();
    expect(tokenClass(true)).not.toBeNull();
  });

  it('hasError changes the built extension (destructive accent instead of primary)', () => {
    const normal = buildCodeMirrorTheme({ isDark: false });
    const errored = buildCodeMirrorTheme({ isDark: false, hasError: true });
    expect(errored).not.toEqual(normal);
  });

  it('does not throw with or without hasError, in either color scheme', () => {
    expect(() => buildCodeMirrorTheme({ isDark: false, hasError: true })).not.toThrow();
    expect(() => buildCodeMirrorTheme({ isDark: true, hasError: true })).not.toThrow();
    expect(() => buildCodeMirrorTheme({ isDark: false })).not.toThrow();
  });
});
