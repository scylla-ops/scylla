import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import type { MarketItem } from '../../../domain/structs/market-item.struct.ts';
import MarketItemList from './MarketItemList.svelte';

const item = (overrides: Partial<MarketItem> = {}): MarketItem => ({
  provider: 'scylla',
  title: 'Hello world',
  descrption: 'a starter pipeline template',
  ...overrides,
});

describe('MarketItemList', () => {
  it('renders one card per item', () => {
    render(MarketItemList, {
      items: [item(), item({ title: 'Deploy to staging' })],
      filter: '',
    });

    expect(screen.getByText('Hello world')).toBeInTheDocument();
    expect(screen.getByText('Deploy to staging')).toBeInTheDocument();
  });

  it('keeps only the items whose title or provider matches the filter', () => {
    render(MarketItemList, {
      items: [item(), item({ title: 'Deploy to staging', provider: 'acme' })],
      filter: 'acme',
    });

    expect(screen.getByText('Deploy to staging')).toBeInTheDocument();
    expect(screen.queryByText('Hello world')).not.toBeInTheDocument();
  });

  it('renders nothing rather than empty nodes when no item matches', () => {
    render(MarketItemList, { items: [item()], filter: 'nothing like this' });

    expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
  });

  it('survives the list not having arrived yet', () => {
    render(MarketItemList, { items: undefined, filter: '' });

    expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
  });
});
