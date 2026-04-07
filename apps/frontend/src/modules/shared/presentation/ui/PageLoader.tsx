import { motion } from 'framer-motion';

export const PageLoader = () => {
  return (
    <div className='flex items-center justify-center h-full w-full'>
      <motion.div
        className='flex flex-col items-center gap-4'
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.2 }}
      >
        <div className='relative'>
          <motion.div
            className='w-12 h-12 border-4 border-primary/20 border-t-primary rounded-full'
            animate={{ rotate: 360 }}
            transition={{
              duration: 1,
              repeat: Infinity,
              ease: 'linear',
            }}
          />
        </div>
        <p className='text-sm text-muted-foreground'>Chargement...</p>
      </motion.div>
    </div>
  );
};


