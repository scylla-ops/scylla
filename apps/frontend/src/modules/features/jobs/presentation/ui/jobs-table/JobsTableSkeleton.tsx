import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { ListCard, type ListCardSection } from '@shared/presentation/ui/ListCard.tsx';

const skeletonSections: ListCardSection[] = [
  {
    width: '15%',
    className: 'flex items-center gap-3 shrink-0',
    content: (
      <div className='flex items-center gap-2'>
        <Skeleton className='w-5 h-5 rounded-full' />
        <Skeleton className='w-16 h-5 rounded' />
      </div>
    ),
  },
  {
    width: '20%',
    className: 'flex items-center gap-2 shrink-0',
    content: <Skeleton className='w-32 h-4' />,
  },
  {
    width: '30%',
    className: 'flex items-center justify-center shrink-0',
    content: <Skeleton className='w-full h-6 rounded-full' />,
  },
  {
    width: '15%',
    className: 'flex justify-center items-center shrink-0',
    content: <Skeleton className='w-16 h-4' />,
  },
  {
    width: '15%',
    className: 'flex justify-center items-center shrink-0',
    content: <Skeleton className='w-16 h-4' />,
  },
  {
    className: 'flex justify-center items-center gap-1 flex-1 min-w-[48px]',
    noSeparator: true,
    content: (
      <div className='flex gap-2'>
        <Skeleton className='w-8 h-8 rounded-full' />
        <Skeleton className='w-8 h-8 rounded-full' />
      </div>
    ),
  },
];

/**
 * Skeleton loader for the jobs table
 */
const JobsTableSkeleton = () => {
  return (
    <div className='flex flex-col gap-3'>
      {[...Array(6)].map((_, index) => (
        <ListCard key={index} sections={skeletonSections} />
      ))}
    </div>
  );
};

export default JobsTableSkeleton;
