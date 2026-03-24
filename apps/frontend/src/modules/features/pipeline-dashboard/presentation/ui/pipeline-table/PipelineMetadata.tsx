import { Clock } from 'lucide-react';

type PipelineMetadataProps = {
  duration?: string;
  lastRun?: string;
};

/**
 * Displays for the metadata of a pipeline
 */
export const PipelineMetadata = ({
  duration = '1m 12s',
  lastRun = '2m ago',
}: PipelineMetadataProps) => {
  return (
    <>
      <div className='flex items-center gap-1.5'>
        <Clock className='w-3.5 h-3.5' />
        <span>{duration}</span>
      </div>
      <span className='text-xs italic truncate'>{lastRun}</span>
    </>
  );
};
