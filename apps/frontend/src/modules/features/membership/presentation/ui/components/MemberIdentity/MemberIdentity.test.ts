import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render, textSnippet } from '@/test/render.svelte.ts';
import MemberIdentity from './MemberIdentity.svelte';

describe('MemberIdentity', () => {
  it('shows the member name', () => {
    render(MemberIdentity, { name: 'alice' });
    expect(screen.getByText('alice')).toBeInTheDocument();
  });

  it('keeps the full name reachable as a title, since the line truncates', () => {
    render(MemberIdentity, { name: 'a-very-long-service-account-name' });
    expect(screen.getByTitle('a-very-long-service-account-name')).toBeInTheDocument();
  });

  it('hides the derived initial from assistive tech — it duplicates the name', () => {
    const { container } = render(MemberIdentity, { name: 'alice' });

    const avatar = container.querySelector('[aria-hidden="true"]');
    expect(avatar).toHaveTextContent('a');
  });

  it('renders no subtitle line when none is given', () => {
    render(MemberIdentity, { name: 'alice' });
    expect(screen.queryByText('2 roles')).not.toBeInTheDocument();
  });

  it('renders the subtitle it is handed', () => {
    render(MemberIdentity, { name: 'alice', subtitle: textSnippet('2 roles') });
    expect(screen.getByText('2 roles')).toBeInTheDocument();
  });

  it('survives an empty name rather than throwing on the initial', () => {
    expect(() => render(MemberIdentity, { name: '' })).not.toThrow();
  });
});
