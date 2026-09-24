import { currentPathname, currentSearch, navigateTo } from '@platform/context';

const NODES_PARAM = 'nodes';

export interface OpenLogPanels {
  /** In execution order, whatever order they were opened in. */
  readonly openNodeIds: string[];
  readonly isWholeJobOpen: boolean;
  /** Adds or removes one node's panel, leaving the other open ones alone. */
  toggleNode: (nodeId: string) => void;
  /**
   * Makes this node the whole selection, closing every other panel — or leaves
   * the whole job showing when given no node.
   */
  selectNode: (nodeId?: string) => void;
}

/**
 * Which log panels the job details page has open, held in the URL rather than
 * in component state so a link can open the page on exactly the panels it names.
 *
 * `?nodes=build,test` lists the open node panels, and the job as a whole is what
 * shows when that list is empty — the two are exclusive, the whole job being the
 * page at rest rather than a panel competing with the nodes for room. So a link
 * naming nodes (`?nodes=build`, what `goToJobDetails` writes) opens on those
 * nodes alone, a node no execution matches leaves the whole job showing, and
 * closing the last node panel comes back to it.
 *
 * `nodeIds` is a getter: the executions arrive with the job, and a list read
 * once would filter the URL against an empty set on the first paint and drop
 * every panel the link asked for.
 *
 * **The query string is read back from the router, not remembered here.**
 * `useSearchParams` gave the React version that for free; the rune equivalent
 * is `currentSearch()` behind a counter the writes bump, so a `$derived`
 * reading it recomputes. Keeping a private copy of the open set would be a
 * second source of truth for something the URL already holds.
 *
 * There is no location subscription, and none is needed: every write here
 * replaces the entry rather than pushing one, so opening and closing panels
 * leaves no in-page history to step back through. Back from this page goes to
 * the page before it, which unmounts all of this.
 */
export const createOpenLogPanels = (nodeIds: () => readonly string[]): OpenLogPanels => {
  // Not the panel set — just "the URL may have moved". The set itself is always
  // derived from `currentSearch()`, which is what keeps the two in step.
  let revision = $state(0);

  const openNodeIds = $derived.by(() => {
    void revision;
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- parses the query string once, never state
    const requested = (new URLSearchParams(currentSearch()).get(NODES_PARAM) ?? '').split(',');
    return nodeIds().filter(id => requested.includes(id));
  });

  const write = (nextNodeIds: readonly string[]) => {
    const ordered = nodeIds().filter(id => nextNodeIds.includes(id));
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- parses the query string once, never state
    const params = new URLSearchParams(currentSearch());

    if (ordered.length > 0) params.set(NODES_PARAM, ordered.join(','));
    else params.delete(NODES_PARAM);

    const query = params.toString();
    navigateTo(`${currentPathname()}${query ? `?${query}` : ''}`, { replace: true });
    revision += 1;
  };

  return {
    get openNodeIds() {
      return openNodeIds;
    },
    get isWholeJobOpen() {
      return openNodeIds.length === 0;
    },

    toggleNode: (nodeId: string) =>
      write(
        openNodeIds.includes(nodeId)
          ? openNodeIds.filter(id => id !== nodeId)
          : [...openNodeIds, nodeId],
      ),

    selectNode: (nodeId?: string) => write(nodeId === undefined ? [] : [nodeId]),
  };
};
