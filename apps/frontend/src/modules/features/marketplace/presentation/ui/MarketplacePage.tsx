import { MarketItemList } from '@/modules/features/marketplace/presentation/ui/MarketItemList.tsx';
import { MarketItemCardSkeleton } from '@/modules/features/marketplace/presentation/ui/MarketItemCardSkeleton.tsx';
import { useMarketplace } from '@/modules/features/marketplace/presentation/hooks/useMarketplace.ts';
import { useFilterStore } from '@/modules/features/marketplace/presentation/stores/useFilter.ts';
import { useDelayedLoading } from '@/modules/shared/presentation/hooks/useDelayedLoading.ts';

export const MarketplacePage = () => {
  const { data, isLoading, isError, error } = useMarketplace();
  const filter = useFilterStore(state => state.filter);
  const showSkeleton = useDelayedLoading(400);

  if (isLoading && !showSkeleton) return <></>;

  if (isLoading && showSkeleton) {
    return (
      <div className='flex flex-col bg-background'>
        <div className='flex flex-row flex-wrap gap-4 w-full h-full p-2'>
          {Array.from({ length: 6 }).map((_, i) => (
            <MarketItemCardSkeleton key={i} />
          ))}
        </div>
      </div>
    );
  }

  if (isError) return <p>Error : {error.message}</p>;

  return (
    <div className='flex flex-col bg-background'>
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <MarketItemList items={data} filter={filter} />
      </div>
    </div>
  );
};
export default MarketplacePage;
