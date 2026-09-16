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
 * and an id no node matches never survives a write.
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

  /** Drops every node panel, which is what leaves the whole job showing. */
  const showWholeJob = () => write([]);

  /** Opens the panel a link points at, on top of whatever is already open. */
  const openPanel = (nodeId?: string) => {
    if (nodeId === undefined) {
      showWholeJob();
      return;
    }
    if (openNodeIds.includes(nodeId)) return;

    write([...openNodeIds, nodeId]);
  };

  return { openNodeIds, isWholeJobOpen, toggleNode, openPanel, showWholeJob };
};
