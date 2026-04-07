import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';

export const PipelineTableSkeleton = () => {
  return (
    <div className='space-y-3 animate-pulse'>
      {/* Header skeleton */}
      <div className='flex items-center gap-4 px-4 py-3 border-b border-border'>
        <Skeleton className='h-5 w-48' />
        <Skeleton className='h-5 w-32' />
        <Skeleton className='h-5 w-24' />
        <Skeleton className='h-5 w-20' />
      </div>

      {/* Rows skeleton */}
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          key={i}
          className='flex items-center gap-4 px-4 py-4 border-b border-border/50'
        >
          <div className='flex items-center gap-3 flex-1'>
            <Skeleton className='h-8 w-8 rounded' />
            <div className='space-y-2 flex-1'>
              <Skeleton className='h-5 w-40' />
              <Skeleton className='h-3 w-56' />
            </div>
          </div>
          <Skeleton className='h-6 w-20 rounded-full' />
          <Skeleton className='h-4 w-24' />
          <Skeleton className='h-8 w-8 rounded' />
        </div>
      ))}
    </div>
  );
};

export default PipelineTableSkeleton;

