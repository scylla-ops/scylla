import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import AvatarFixture from './avatar.fixture.svelte';

describe('Avatar', () => {
  it('shows the fallback while the image has not loaded', () => {
    // jsdom never loads an image, which is also what a broken avatar URL does
    // in production — the fallback is the state users actually see most.
    render(AvatarFixture, { src: 'https://example.invalid/ada.png' });

    expect(screen.getByText('AL')).toBeInTheDocument();
  });

  it('shows the fallback when there is no image at all', () => {
    render(AvatarFixture);

    expect(screen.getByText('AL')).toBeInTheDocument();
    expect(screen.queryByRole('img')).toBeNull();
  });
});
