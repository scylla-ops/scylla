import { useState, useEffect } from 'react';

/**
 * Hook pour retarder l'affichage du skeleton/loader
 * Évite les flashs de contenu de chargement pour les requêtes rapides
 * 
 * @param delay - Délai en ms avant d'afficher le loader (défaut: 400ms)
 * @returns true si le délai est écoulé
 */
export const useDelayedLoading = (delay: number = 400): boolean => {
  const [showLoading, setShowLoading] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setShowLoading(true);
    }, delay);

    return () => clearTimeout(timer);
  }, [delay]);

  return showLoading;
};


