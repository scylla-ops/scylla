import { MarketItemList } from '@/modules/features/marketplace/presentation/ui/MarketItemList.tsx';
import { useMarketplace } from '@/modules/features/marketplace/presentation/hooks/useMarketplace.ts';
import { useFilterStore } from '@/modules/features/marketplace/presentation/stores/useFilter.ts';

export const MarketplacePage = () => {
  const { data, isLoading, isError, error } = useMarketplace();
  const filter = useFilterStore(state => state.filter);

  if (isLoading) return <p>Loading...</p>;
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
