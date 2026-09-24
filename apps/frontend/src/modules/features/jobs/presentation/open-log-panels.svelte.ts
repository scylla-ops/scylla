import { currentPathname, currentSearch, navigateTo } from '@platform/context';

const NODES_PARAM = 'nodes';

export interface OpenLogPanels {
  /** In execution order. */
  readonly openNodeIds: string[];
  readonly isWholeJobOpen: boolean;
  toggleNode: (nodeId: string) => void;
  /** Opens only this node, or the whole job when given none. */
  selectNode: (nodeId?: string) => void;
}

/**
 * The open log panels, kept in the URL (`?nodes=build,test`) so a link can open
 * them. No node open means the whole job shows. `nodeIds` is a getter: the nodes
 * arrive with the job. The query string is read back from the router, never copied.
 */
export const createOpenLogPanels = (nodeIds: () => readonly string[]): OpenLogPanels => {
  // Bumped by each write so the derived set re-reads `currentSearch()`.
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
