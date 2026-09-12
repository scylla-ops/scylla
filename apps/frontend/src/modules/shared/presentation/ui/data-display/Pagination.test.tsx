import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { Pagination } from './Pagination';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';

const info = (overrides: Partial<PaginationInfo> = {}): PaginationInfo => ({
  page: 1,
  pageSize: 10,
  totalCount: 100,
  totalPages: 10,
  hasNext: true,
  hasPrevious: false,
  ...overrides,
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

// The page-number links are plain `<a>` tags with no `href`, so they carry no
// implicit ARIA role - unlike Previous/Next, which get one via their
// aria-label. Distinguish them from Previous/Next by the absence of one.
const pageLinks = (container: HTMLElement) =>
  Array.from(container.querySelectorAll('a[data-slot="pagination-link"]:not([aria-label])')).map(
    el => el.textContent,
  );

describe('Pagination', () => {
  it('lists every page when there are 7 or fewer', () => {
    const { container } = renderWithI18n(
      <Pagination paginationInfo={info({ totalPages: 5, page: 1 })} onPageChange={vi.fn()} />,
    );
    expect(pageLinks(container)).toEqual(['1', '2', '3', '4', '5']);
  });

  it('shows the first 4 pages + an ellipsis + the last page near the start', () => {
    const { container } = renderWithI18n(
      <Pagination paginationInfo={info({ totalPages: 10, page: 2 })} onPageChange={vi.fn()} />,
    );
    expect(pageLinks(container)).toEqual(['1', '2', '3', '4', '10']);
  });

  it('shows the last 4 pages + an ellipsis + the first page near the end', () => {
    const { container } = renderWithI18n(
      <Pagination paginationInfo={info({ totalPages: 10, page: 9 })} onPageChange={vi.fn()} />,
    );
    expect(pageLinks(container)).toEqual(['1', '7', '8', '9', '10']);
  });

  it('shows first, a window around the current page, and last, in the middle', () => {
    const { container } = renderWithI18n(
      <Pagination paginationInfo={info({ totalPages: 10, page: 5 })} onPageChange={vi.fn()} />,
    );
    expect(pageLinks(container)).toEqual(['1', '4', '5', '6', '10']);
  });

  it('marks the current page as active', () => {
    const { container } = renderWithI18n(
      <Pagination paginationInfo={info({ totalPages: 5, page: 3 })} onPageChange={vi.fn()} />,
    );
    const active = container.querySelector('a[data-slot="pagination-link"][data-active="true"]');
    expect(active).toHaveTextContent('3');
    expect(active).toHaveAttribute('aria-current', 'page');
  });

  it('calls onPageChange with the clicked page number', async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<Pagination paginationInfo={info({ totalPages: 5, page: 1 })} onPageChange={onPageChange} />);
    await user.click(screen.getByText('3'));
    expect(onPageChange).toHaveBeenCalledWith(3);
  });

  it('disables Previous on the first page and does not call onPageChange when clicked', async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <Pagination paginationInfo={info({ page: 1, hasPrevious: false })} onPageChange={onPageChange} />,
    );
    await user.click(screen.getByLabelText('Go to previous page'));
    expect(onPageChange).not.toHaveBeenCalled();
  });

  it('Next advances to the next page when available', async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <Pagination paginationInfo={info({ page: 2, hasNext: true })} onPageChange={onPageChange} />,
    );
    await user.click(screen.getByLabelText('Go to next page'));
    expect(onPageChange).toHaveBeenCalledWith(3);
  });

  it('shows the "Showing X-Y of Z" summary', () => {
    renderWithI18n(
      <Pagination paginationInfo={info({ page: 2, pageSize: 10, totalCount: 25, totalPages: 3 })} onPageChange={vi.fn()} />,
    );
    expect(screen.getByText('Showing 11-20 of 25')).toBeInTheDocument();
  });

  it('caps the "end" figure at totalCount on the last (partial) page', () => {
    renderWithI18n(
      <Pagination paginationInfo={info({ page: 3, pageSize: 10, totalCount: 25, totalPages: 3 })} onPageChange={vi.fn()} />,
    );
    expect(screen.getByText('Showing 21-25 of 25')).toBeInTheDocument();
  });
});
