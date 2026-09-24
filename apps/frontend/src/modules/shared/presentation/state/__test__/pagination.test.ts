import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createPagination } from '../pagination.svelte.ts';

/**
 * jsdom, but only for `document.createElement` and a driveable ResizeObserver —
 * the action needs a real element to attach to. Everything else here is plain
 * state.
 */
class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  callback: ResizeObserverCallback;
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
    ResizeObserverMock.instances.push(this);
  }

  fire(height: number) {
    this.callback([{ contentRect: { height } } as unknown as ResizeObserverEntry], this);
  }
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
});

/** What `use:measure` does, called by hand. */
const attach = (measure: ReturnType<typeof createPagination>['measure'], height = 0) => {
  const node = document.createElement('div');
  vi.spyOn(node, 'clientHeight', 'get').mockReturnValue(height);
  return measure(node);
};

describe('createPagination', () => {
  it('defaults to page 1 and a flat page size without a responsive container', () => {
    const pagination = createPagination();

    expect(pagination.page).toBe(1);
    expect(pagination.pageSize).toBe(10);
    expect(pagination.paginationParams).toEqual({ page: 1, pageSize: 10 });
  });

  it('is ready right away without a responsive container', () => {
    expect(createPagination().isPageSizeReady).toBe(true);
  });

  it('is not ready until the responsive container has been measured', () => {
    const pagination = createPagination({ responsive: true });
    expect(pagination.isPageSizeReady).toBe(false);

    attach(pagination.measure);

    expect(pagination.isPageSizeReady).toBe(true);
  });

  it('is ready right away when an explicit initialPageSize opts out of measuring', () => {
    const pagination = createPagination({ responsive: true, initialPageSize: 15 });

    expect(pagination.isPageSizeReady).toBe(true);
  });

  it('sizes to the responsive container once attached, instead of the flat default', () => {
    const pagination = createPagination({ responsive: true, rowHeight: 60, headerHeight: 40 });

    attach(pagination.measure, 640);
    expect(pagination.pageSize).toBe(10);

    ResizeObserverMock.instances[0].fire(1240);
    expect(pagination.pageSize).toBe(20);
  });

  it('knows its size on attach, without waiting for the observer to fire', () => {
    const pagination = createPagination({ responsive: true, rowHeight: 60, headerHeight: 40 });

    attach(pagination.measure, 1240);

    // Nothing has fired yet — the height came from `clientHeight`, which is what
    // stops the first paint happening at a placeholder size.
    expect(pagination.pageSize).toBe(20);
  });

  it('lets an explicit initialPageSize win over a responsive container', () => {
    const pagination = createPagination({ responsive: true, initialPageSize: 15 });

    attach(pagination.measure, 640);

    expect(pagination.pageSize).toBe(15);
  });

  it('honors the initial page and page size', () => {
    const pagination = createPagination({ initialPage: 3, initialPageSize: 25 });

    expect(pagination.page).toBe(3);
    expect(pagination.pageSize).toBe(25);
  });

  it('clamps a page below 1 up to 1', () => {
    const pagination = createPagination();

    pagination.setPage(0);
    expect(pagination.page).toBe(1);

    pagination.setPage(-5);
    expect(pagination.page).toBe(1);
  });

  it('clamps a page size to [1, 100] and returns to page 1', () => {
    const pagination = createPagination();
    pagination.setPage(4);

    pagination.setPageSize(500);
    expect(pagination.pageSize).toBe(100);
    expect(pagination.page).toBe(1);

    pagination.setPageSize(0);
    expect(pagination.pageSize).toBe(1);
  });

  it('pins the size once the user picks one, even on a responsive container', () => {
    const pagination = createPagination({ responsive: true, rowHeight: 60, headerHeight: 40 });
    attach(pagination.measure, 640);

    pagination.setPageSize(25);
    ResizeObserverMock.instances[0].fire(1240);

    expect(pagination.pageSize).toBe(25);
  });

  it('has no paginationInfo until the server has reported some', () => {
    expect(createPagination().paginationInfo).toBeUndefined();
  });

  it('derives hasNext/hasPrevious from the current page, not from the server', () => {
    const pagination = createPagination({ initialPageSize: 10 });
    pagination.updatePaginationInfo({
      totalCount: 30,
      totalPages: 3,
      page: 1,
      pageSize: 10,
      hasNext: false,
      hasPrevious: false,
    });

    expect(pagination.paginationInfo).toMatchObject({ page: 1, hasNext: true, hasPrevious: false });

    pagination.setPage(3);

    expect(pagination.paginationInfo).toMatchObject({ page: 3, hasNext: false, hasPrevious: true });
  });

  it('snaps back to the last page when the current one no longer exists', () => {
    const pagination = createPagination({ initialPage: 5, initialPageSize: 10 });

    pagination.updatePaginationInfo({
      totalCount: 20,
      totalPages: 2,
      page: 5,
      pageSize: 10,
      hasNext: false,
      hasPrevious: true,
    });

    expect(pagination.page).toBe(2);
  });

  it('leaves the page alone when totalPages is 0 — an empty result set, not an overshoot', () => {
    const pagination = createPagination({ initialPage: 3, initialPageSize: 10 });

    pagination.updatePaginationInfo({
      totalCount: 0,
      totalPages: 0,
      page: 3,
      pageSize: 10,
      hasNext: false,
      hasPrevious: true,
    });

    expect(pagination.page).toBe(3);
  });

  it('stops observing when the container goes away', () => {
    const pagination = createPagination({ responsive: true });
    const handle = attach(pagination.measure, 640);

    handle?.destroy?.();

    expect(ResizeObserverMock.instances[0].disconnect).toHaveBeenCalled();
    expect(pagination.isPageSizeReady).toBe(false);
  });
});
