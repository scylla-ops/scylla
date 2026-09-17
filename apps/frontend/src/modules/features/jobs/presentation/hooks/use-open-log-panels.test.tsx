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
  it('opens on the whole job when the URL names no node', () => {
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

  it('ignores the whole-job flag an older link may still carry', () => {
    const { result } = renderPanels('?nodes=build&whole=1');

    expect(result.current.openNodeIds).toEqual(['build']);
    expect(result.current.isWholeJobOpen).toBe(false);
  });

  it('opens a second node without closing the first', () => {
    const { result } = renderPanels('?nodes=build');

    act(() => result.current.toggleNode('deploy'));

    expect(result.current.openNodeIds).toEqual(['build', 'deploy']);
    expect(result.current.search).toBe('?nodes=build%2Cdeploy');
  });

  it('closes a node that is already open, leaving the others alone', () => {
    const { result } = renderPanels('?nodes=build,deploy');

    act(() => result.current.toggleNode('build'));

    expect(result.current.openNodeIds).toEqual(['deploy']);
    expect(result.current.isWholeJobOpen).toBe(false);
  });

  it('comes back to the whole job once the last node is closed', () => {
    const { result } = renderPanels('?nodes=build');

    act(() => result.current.toggleNode('build'));

    expect(result.current.isWholeJobOpen).toBe(true);
    expect(result.current.openNodeIds).toEqual([]);
    expect(result.current.search).toBe('');
  });

  it('selects one node in place of everything that was open', () => {
    const { result } = renderPanels('?nodes=build,deploy');

    act(() => result.current.selectNode('test'));

    expect(result.current.openNodeIds).toEqual(['test']);
    expect(result.current.search).toBe('?nodes=test');
  });

  it('selects the node a link points at, which is what hides the whole job', () => {
    const { result } = renderPanels();

    act(() => result.current.selectNode('test'));

    expect(result.current.openNodeIds).toEqual(['test']);
    expect(result.current.isWholeJobOpen).toBe(false);
  });

  it('selecting the open node again leaves it open, rather than closing it', () => {
    const { result } = renderPanels('?nodes=test');

    act(() => result.current.selectNode('test'));

    expect(result.current.openNodeIds).toEqual(['test']);
  });

  it('selects no node at all to come back to the whole job', () => {
    const { result } = renderPanels('?nodes=build,test');

    act(() => result.current.selectNode());

    expect(result.current.isWholeJobOpen).toBe(true);
    expect(result.current.search).toBe('');
  });

  it('never writes an id no execution matches back to the URL', () => {
    const { result } = renderPanels('?nodes=ghost');

    act(() => result.current.selectNode('build'));

    expect(result.current.search).toBe('?nodes=build');
  });

  it('keeps unrelated search params across a selection', () => {
    const { result } = renderPanels('?tab=summary');

    act(() => result.current.selectNode('build'));

    expect(result.current.search).toContain('tab=summary');
  });
});
