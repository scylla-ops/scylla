import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { installTestNavigator } from '@/test/navigator.ts';
import { createOpenLogPanels } from '../open-log-panels.svelte.ts';

const NODES = ['build', 'test', 'deploy'];
const PATHNAME = '/acme/projects/p1/pipelines/pipe-1/jobs/job-1';

let navigator: ReturnType<typeof installTestNavigator>;

/** Installs a navigator already sitting on `search`, as an incoming link would. */
const openedWith = (search: string) => {
  navigator = installTestNavigator({ pathname: PATHNAME, search });
};

beforeEach(() => openedWith(''));
afterEach(() => navigator.restore());

const panels = (nodeIds: readonly string[] = NODES) => createOpenLogPanels(() => nodeIds);

describe('createOpenLogPanels', () => {
  it('opens on the whole job when the URL names no node', () => {
    const cleanup = $effect.root(() => {
      const state = panels();
      expect(state.openNodeIds).toEqual([]);
      expect(state.isWholeJobOpen).toBe(true);
    });
    cleanup();
  });

  it('opens every node the URL lists, and only those', () => {
    openedWith('?nodes=build,deploy');

    const cleanup = $effect.root(() => {
      const state = panels();
      expect(state.openNodeIds).toEqual(['build', 'deploy']);
      expect(state.isWholeJobOpen).toBe(false);
    });
    cleanup();
  });

  it('reads the panels back in execution order, not in the order the URL lists them', () => {
    openedWith('?nodes=deploy,build');

    const cleanup = $effect.root(() => {
      expect(panels().openNodeIds).toEqual(['build', 'deploy']);
    });
    cleanup();
  });

  it('falls back to the whole job for an id no execution matches', () => {
    openedWith('?nodes=ghost');

    const cleanup = $effect.root(() => {
      expect(panels().isWholeJobOpen).toBe(true);
    });
    cleanup();
  });

  it('filters against the executions as they arrive, not against the empty first render', () => {
    openedWith('?nodes=build');

    const cleanup = $effect.root(() => {
      // The job is still loading: nothing matches yet.
      let nodeIds = $state<string[]>([]);
      const state = createOpenLogPanels(() => nodeIds);
      expect(state.openNodeIds).toEqual([]);

      nodeIds = NODES;
      flushSync();

      // A list read once would have dropped the panel the link asked for.
      expect(state.openNodeIds).toEqual(['build']);
    });
    cleanup();
  });

  it('opens a second node without closing the first', () => {
    openedWith('?nodes=build');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.toggleNode('deploy');
      flushSync();

      expect(state.openNodeIds).toEqual(['build', 'deploy']);
    });
    cleanup();
  });

  it('closes a node that is already open, leaving the others alone', () => {
    openedWith('?nodes=build,test,deploy');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.toggleNode('test');
      flushSync();

      expect(state.openNodeIds).toEqual(['build', 'deploy']);
    });
    cleanup();
  });

  it('comes back to the whole job once the last node is closed', () => {
    openedWith('?nodes=build');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.toggleNode('build');
      flushSync();

      expect(state.isWholeJobOpen).toBe(true);
      // Written away entirely rather than left as an empty value.
      expect(navigator.navigate).toHaveBeenCalledWith(PATHNAME, { replace: true });
    });
    cleanup();
  });

  it('selects one node in place of everything that was open', () => {
    openedWith('?nodes=build,test');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.selectNode('deploy');
      flushSync();

      expect(state.openNodeIds).toEqual(['deploy']);
    });
    cleanup();
  });

  it('selecting the open node again leaves it open, rather than closing it', () => {
    openedWith('?nodes=build');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.selectNode('build');
      flushSync();

      expect(state.openNodeIds).toEqual(['build']);
    });
    cleanup();
  });

  it('selects no node at all to come back to the whole job', () => {
    openedWith('?nodes=build,test');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.selectNode();
      flushSync();

      expect(state.isWholeJobOpen).toBe(true);
    });
    cleanup();
  });

  it('never writes an id no execution matches back to the URL', () => {
    openedWith('?nodes=ghost');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.toggleNode('build');
      flushSync();

      expect(navigator.navigate).toHaveBeenCalledWith(`${PATHNAME}?nodes=build`, {
        replace: true,
      });
    });
    cleanup();
  });

  it('keeps unrelated search params across a selection', () => {
    openedWith('?tab=logs&nodes=build');

    const cleanup = $effect.root(() => {
      const state = panels();
      state.selectNode('test');
      flushSync();

      const [url] = navigator.navigate.mock.calls.at(-1) as [string];
      expect(new URLSearchParams(url.slice(url.indexOf('?'))).get('tab')).toBe('logs');
    });
    cleanup();
  });

  it('replaces the history entry rather than pushing one per panel toggled', () => {
    const cleanup = $effect.root(() => {
      const state = panels();
      state.toggleNode('build');
      flushSync();

      expect(navigator.navigate).toHaveBeenCalledWith(expect.any(String), { replace: true });
    });
    cleanup();
  });
});
