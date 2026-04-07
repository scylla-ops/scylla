import { Card, CardContent, CardHeader } from '@shadcn';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';

export const ProjectCardSkeleton = () => {
  return (
    <Card className='h-full animate-pulse'>
      <CardHeader className='space-y-0 pb-3'>
        <div className='flex items-start justify-between gap-3'>
          <div className='flex items-center gap-3 min-w-0 flex-1'>
            <Skeleton className='h-10 w-10 rounded-lg' />
            <Skeleton className='h-6 w-32' />
          </div>
        </div>
      </CardHeader>

      <CardContent className='space-y-3'>
        <div className='space-y-2'>
          <Skeleton className='h-4 w-full' />
          <Skeleton className='h-4 w-3/4' />
        </div>

        <div className='pt-2 border-t border-border/50'>
          <div className='flex items-center justify-between'>
            <Skeleton className='h-3 w-16' />
            <Skeleton className='h-3 w-20' />
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

export default ProjectCardSkeleton;

