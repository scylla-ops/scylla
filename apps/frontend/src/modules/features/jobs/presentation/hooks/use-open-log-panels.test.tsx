import type { ReactNode } from 'react';
import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { useOpenLogPanels } from './use-open-log-panels';

const NODE_IDS = ['build', 'test', 'deploy'];

const renderPanels = (search = '', nodeIds: string[] = NODE_IDS) => {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <MemoryRouter initialEntries={[`/jobs/job-1${search}`]}>{children}</MemoryRouter>
  );

  return renderHook(() => ({ ...useOpenLogPanels(nodeIds), search: useLocation().search }), {
    wrapper,
  });
};

describe('useOpenLogPanels', () => {
  it('opens on the whole job alone when the URL names nothing', () => {
    const { result } = renderPanels();

    expect(result.current.isWholeJobOpen).toBe(true);
    expect(result.current.openNodeIds).toEqual([]);
  });

  it('opens every node the URL lists, and only those', () => {
    const { result } = renderPanels('?nodes=build,deploy');

    expect(result.current.openNodeIds).toEqual(['build', 'deploy']);
    expect(result.current.isWholeJobOpen).toBe(false);
  });

  it('reads the panels back in execution order, not in the order the URL lists them', () => {
    const { result } = renderPanels('?nodes=deploy,build');

    expect(result.current.openNodeIds).toEqual(['build', 'deploy']);
  });

  it('falls back to the whole job for an id no execution matches', () => {
    const { result } = renderPanels('?nodes=ghost');

    expect(result.current.openNodeIds).toEqual([]);
    expect(result.current.isWholeJobOpen).toBe(true);
  });

  it('keeps the whole job open when the URL says so alongside nodes', () => {
    const { result } = renderPanels('?nodes=build&whole=1');

    expect(result.current.openNodeIds).toEqual(['build']);
    expect(result.current.isWholeJobOpen).toBe(true);
  });

  it('opens a second node without closing the first', () => {
    const { result } = renderPanels('?nodes=build&whole=0');

    act(() => result.current.togglePanel('deploy'));

    expect(result.current.openNodeIds).toEqual(['build', 'deploy']);
    expect(result.current.search).toBe('?nodes=build%2Cdeploy&whole=0');
  });

  it('closes a node that is already open, leaving the others alone', () => {
    const { result } = renderPanels('?nodes=build,deploy&whole=0');

    act(() => result.current.togglePanel('build'));

    expect(result.current.openNodeIds).toEqual(['deploy']);
  });

  it('toggles the whole job panel on its own', () => {
    const { result } = renderPanels('?nodes=build&whole=0');

    act(() => result.current.togglePanel());

    expect(result.current.isWholeJobOpen).toBe(true);
    expect(result.current.openNodeIds).toEqual(['build']);
  });

  it('leaves nothing open once the last panel is closed', () => {
    const { result } = renderPanels();

    act(() => result.current.togglePanel());

    expect(result.current.isWholeJobOpen).toBe(false);
    expect(result.current.openNodeIds).toEqual([]);
    expect(result.current.search).toBe('?whole=0');
  });

  it('adds the node a link points at without closing what is open', () => {
    const { result } = renderPanels();

    act(() => result.current.openPanel('test'));

    expect(result.current.openNodeIds).toEqual(['test']);
    expect(result.current.isWholeJobOpen).toBe(true);
  });

  it('opening a node twice is a no-op, not a close', () => {
    const { result } = renderPanels('?nodes=test&whole=0');

    act(() => result.current.openPanel('test'));

    expect(result.current.openNodeIds).toEqual(['test']);
  });

  it('never writes an id no execution matches back to the URL', () => {
    const { result } = renderPanels('?nodes=ghost');

    act(() => result.current.openPanel('build'));

    expect(result.current.search).toBe('?nodes=build&whole=1');
  });

  it('keeps unrelated search params across a toggle', () => {
    const { result } = renderPanels('?tab=summary');

    act(() => result.current.openPanel('build'));

    expect(result.current.search).toContain('tab=summary');
  });
});
