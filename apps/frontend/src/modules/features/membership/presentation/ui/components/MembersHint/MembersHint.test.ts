import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render, textSnippet } from '@/test/render.svelte.ts';
import MembersHint from './MembersHint.svelte';

describe('MembersHint', () => {
  it('renders the footnote it is given', () => {
    render(MembersHint, {
      children: textSnippet('Owners are listed even without a role here.'),
    });

    expect(screen.getByText('Owners are listed even without a role here.')).toBeInTheDocument();
  });
});
