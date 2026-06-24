import { useEffect, useState } from 'react';

/**
 * Returns a ticking timestamp that can be used to force live UI updates.
 * Useful for components that need to show elapsed time while something is running.
 */
export const useNow = (enabled = true, intervalMs = 1000) => {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!enabled) return;

    const id = window.setInterval(() => {
      setNow(Date.now());
    }, intervalMs);

    return () => window.clearInterval(id);
  }, [enabled, intervalMs]);

  return now;
};
