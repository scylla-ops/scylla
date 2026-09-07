import { motion } from 'framer-motion';
import iconScylla from '@/assets/icon_scylla.png';

export const ScyllaLoadingScreen = () => (
  <div className='flex items-center justify-center h-screen w-screen bg-background'>
    <motion.img
      src={iconScylla}
      alt='Scylla'
      className='h-28 w-28'
      animate={{ rotate: [0, 270, 360] }}
      transition={{
        duration: 1.8,
        repeat: Infinity,
        times: [0, 0.4, 1],
        ease: 'easeInOut',
      }}
    />
  </div>
);
