import { motion } from 'framer-motion';
import { useEffect, useState } from 'react';
import iconScylla from '@/assets/icon_scylla.png';

/**
 * How long a load may take before it is worth telling the user about. Under
 * this, painting the logo and removing it in the same breath reads as a glitch
 * rather than as progress — so the screen stays empty and a fast load shows
 * nothing at all.
 */
const LOGO_DELAY_MS = 300;

export const ScyllaLoadingScreen = () => {
  const [showLogo, setShowLogo] = useState(false);

  // The timer is the outside system here — no React state says "300ms have
  // passed". Callers render this screen while their query is pending, so a
  // fast load unmounts it first and the cleanup cancels the timer.
  useEffect(() => {
    const timer = setTimeout(() => setShowLogo(true), LOGO_DELAY_MS);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div className='flex items-center justify-center h-screen w-screen bg-background'>
      {showLogo && (
        <motion.img
          src={iconScylla}
          alt='Scylla'
          className='h-28 w-28'
          initial={{ opacity: 0 }}
          // Fading in rather than cutting in keeps the boundary soft: a load
          // that ends just after the delay never reaches full opacity.
          animate={{ opacity: 1, rotate: [0, 270, 360] }}
          transition={{
            opacity: { duration: 0.2, ease: 'easeOut' },
            rotate: {
              duration: 1.8,
              repeat: Infinity,
              times: [0, 0.4, 1],
              ease: 'easeInOut',
            },
          }}
        />
      )}
    </div>
  );
};
