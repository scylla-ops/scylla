import { Card, CardContent, CardFooter, CardHeader } from '@shadcn';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';

export const MarketItemCardSkeleton = () => {
  return (
    <Card className='flex-1 min-w-[300px] max-w-[400px] min-h-[250px] flex flex-col gap-2 animate-pulse'>
      <CardHeader className='flex items-center justify-between'>
        <Skeleton className='w-24 h-24 rounded-lg' />
        <Skeleton className='h-4 w-16' />
      </CardHeader>

      <CardContent className='flex-1 space-y-3'>
        <Skeleton className='h-7 w-3/4' />
        <div className='space-y-2'>
          <Skeleton className='h-4 w-full' />
          <Skeleton className='h-4 w-2/3' />
        </div>
      </CardContent>

      <CardFooter className='flex justify-end gap-2'>
        <Skeleton className='h-9 w-24 rounded-md' />
        <Skeleton className='h-9 w-24 rounded-md' />
      </CardFooter>
    </Card>
  );
};
