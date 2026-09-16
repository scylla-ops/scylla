import { useSearchParams } from 'react-router-dom';

const NODES_PARAM = 'nodes';
const WHOLE_JOB_PARAM = 'whole';

/**
 * Which log panels the job details page has open, held in the URL rather than in
 * component state so a link can open the page on exactly the panels it names.
 *
 * `?nodes=build,test` lists the open node panels and `?whole=1|0` the job-wide
 * one. With neither, the page opens on the whole job alone — so a link naming
 * only nodes (`?nodes=build`, what `goToJobDetails` writes) opens on those
 * nodes alone, and a node no execution matches leaves the whole job showing.
 *
 * Panels are read back in execution order, whatever order they were opened in,
 * and an id no node matches never survives a write.
 */
export const useOpenLogPanels = (nodeIds: readonly string[]) => {
  const [searchParams, setSearchParams] = useSearchParams();

  const requested = (searchParams.get(NODES_PARAM) ?? '').split(',');
  const openNodeIds = nodeIds.filter(id => requested.includes(id));
  const wholeJob = searchParams.get(WHOLE_JOB_PARAM);
  const isWholeJobOpen = wholeJob === null ? openNodeIds.length === 0 : wholeJob === '1';

  const write = (nextNodeIds: readonly string[], nextWholeJob: boolean) => {
    const ordered = nodeIds.filter(id => nextNodeIds.includes(id));
    const params = new URLSearchParams(searchParams);

    if (ordered.length > 0) params.set(NODES_PARAM, ordered.join(','));
    else params.delete(NODES_PARAM);

    // The pristine default — whole job alone — stays a bare URL.
    if (nextWholeJob && ordered.length === 0) params.delete(WHOLE_JOB_PARAM);
    else params.set(WHOLE_JOB_PARAM, nextWholeJob ? '1' : '0');

    setSearchParams(params, { replace: true });
  };

  const togglePanel = (nodeId?: string) => {
    if (nodeId === undefined) {
      write(openNodeIds, !isWholeJobOpen);
      return;
    }

    const next = openNodeIds.includes(nodeId)
      ? openNodeIds.filter(id => id !== nodeId)
      : [...openNodeIds, nodeId];
    write(next, isWholeJobOpen);
  };

  const openPanel = (nodeId?: string) => {
    if (nodeId === undefined) {
      write(openNodeIds, true);
      return;
    }
    if (openNodeIds.includes(nodeId)) return;

    write([...openNodeIds, nodeId], isWholeJobOpen);
  };

  return { openNodeIds, isWholeJobOpen, togglePanel, openPanel };
};
