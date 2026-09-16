import { useSearchParams } from 'react-router-dom';

const NODES_PARAM = 'nodes';

/**
 * Which log panels the job details page has open, held in the URL rather than in
 * component state so a link can open the page on exactly the panels it names.
 *
 * `?nodes=build,test` lists the open node panels, and the job as a whole is what
 * shows when that list is empty — the two are exclusive, the whole job being the
 * page at rest rather than a panel competing with the nodes for room. So a link
 * naming nodes (`?nodes=build`, what `goToJobDetails` writes) opens on those
 * nodes alone, a node no execution matches leaves the whole job showing, and
 * closing the last node panel comes back to it.
 *
 * Panels are read back in execution order, whatever order they were opened in,
 * and an id no node matches never survives a write. The two ways in differ:
 * `selectNode` jumps to one node alone, the way a click on the timeline means
 * "show me this one", while `toggleNode` builds the set up panel by panel.
 */
export const useOpenLogPanels = (nodeIds: readonly string[]) => {
  const [searchParams, setSearchParams] = useSearchParams();

  const requested = (searchParams.get(NODES_PARAM) ?? '').split(',');
  const openNodeIds = nodeIds.filter(id => requested.includes(id));
  const isWholeJobOpen = openNodeIds.length === 0;

  const write = (nextNodeIds: readonly string[]) => {
    const ordered = nodeIds.filter(id => nextNodeIds.includes(id));
    const params = new URLSearchParams(searchParams);

    if (ordered.length > 0) params.set(NODES_PARAM, ordered.join(','));
    else params.delete(NODES_PARAM);

    setSearchParams(params, { replace: true });
  };

  /** Adds or removes one node's panel, leaving the other open ones alone. */
  const toggleNode = (nodeId: string) => {
    write(
      openNodeIds.includes(nodeId)
        ? openNodeIds.filter(id => id !== nodeId)
        : [...openNodeIds, nodeId],
    );
  };

  /**
   * Makes this node the whole selection, closing every other panel — or leaves
   * the whole job showing when given no node.
   */
  const selectNode = (nodeId?: string) => write(nodeId === undefined ? [] : [nodeId]);

  return { openNodeIds, isWholeJobOpen, toggleNode, selectNode };
};
