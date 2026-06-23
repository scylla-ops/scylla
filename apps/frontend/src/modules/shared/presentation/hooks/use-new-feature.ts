import { useState, useEffect, useCallback } from 'react';

const DISMISSED_EVENT = 'new-feature-dismissed';

export const useNewFeature = (key: string) => {
  const storageKey = `new-feature-seen:${key}`;
  const [isNew, setIsNew] = useState(() => !localStorage.getItem(storageKey));

  useEffect(() => {
    const handler = (e: CustomEvent<string>) => {
      if (e.detail === key) setIsNew(false);
    };
    window.addEventListener(DISMISSED_EVENT, handler as EventListener);
    return () => window.removeEventListener(DISMISSED_EVENT, handler as EventListener);
  }, [key]);

  const markSeen = useCallback(() => {
    localStorage.setItem(storageKey, '1');
    setIsNew(false);
    window.dispatchEvent(new CustomEvent(DISMISSED_EVENT, { detail: key }));
  }, [storageKey, key]);

  return { isNew, markSeen };
};
