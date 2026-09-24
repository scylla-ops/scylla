import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, textSnippet } from '@/test/render.svelte.ts';
import Button from '../button.svelte';

/**
 * The primitives are vendored and excluded from coverage, so this is not about
 * hitting lines. It pins the two things the port could silently get wrong: the
 * app-owned attributes the rest of the suite queries by, and the `href` arm that
 * replaces React's `asChild`.
 */
describe('Button', () => {
  it('is found by its accessible role and name', () => {
    render(Button, { children: textSnippet('Save') });

    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
  });

  it('carries the data attributes the suite queries by', () => {
    render(Button, { variant: 'destructive', size: 'sm', children: textSnippet('Delete') });

    const button = screen.getByRole('button', { name: 'Delete' });
    expect(button).toHaveAttribute('data-slot', 'button');
    expect(button).toHaveAttribute('data-variant', 'destructive');
    expect(button).toHaveAttribute('data-size', 'sm');
  });

  it('defaults to the default variant and size, as the React one does', () => {
    render(Button, { children: textSnippet('Go') });

    const button = screen.getByRole('button', { name: 'Go' });
    expect(button).toHaveAttribute('data-variant', 'default');
    expect(button).toHaveAttribute('data-size', 'default');
  });

  it('renders a real anchor when given an href, not a button', () => {
    render(Button, { href: '/pipelines', children: textSnippet('Pipelines') });

    const link = screen.getByRole('link', { name: 'Pipelines' });
    expect(link).toHaveAttribute('href', '/pipelines');
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('fires its click handler', async () => {
    let clicked = 0;
    render(Button, { onclick: () => (clicked += 1), children: textSnippet('Run') });

    await userEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(clicked).toBe(1);
  });

  it('does not fire when disabled', async () => {
    let clicked = 0;
    render(Button, { disabled: true, onclick: () => (clicked += 1), children: textSnippet('Run') });

    await userEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(clicked).toBe(0);
  });
});
