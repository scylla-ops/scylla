// @vitest-environment node
import { describe, it, expect, afterEach } from 'vitest';
import { marketplaceFilter, matchesFilter } from './marketplace-filter.state.svelte.ts';

afterEach(() => marketplaceFilter.set(''));

describe('the marketplace filter', () => {
  it('starts empty, so the catalog is unfiltered on arrival', () => {
    expect(marketplaceFilter.value).toBe('');
  });

  it('remembers what the user typed', () => {
    marketplaceFilter.set('deploy');
    expect(marketplaceFilter.value).toBe('deploy');
  });
});

describe('matchesFilter', () => {
  const item = { title: 'Deploy to staging', provider: 'Acme' };

  it('matches on the title, ignoring case', () => {
    expect(matchesFilter(item, 'DEPLOY')).toBe(true);
  });

  it('matches on the provider too', () => {
    expect(matchesFilter(item, 'acme')).toBe(true);
  });

  it('matches everything when the search is empty', () => {
    expect(matchesFilter(item, '')).toBe(true);
  });

  it('does not match the description — only title and provider are searched', () => {
    expect(matchesFilter(item, 'staging pipeline')).toBe(false);
  });
});
