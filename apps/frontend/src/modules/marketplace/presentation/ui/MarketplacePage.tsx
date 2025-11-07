import { MarketItemList } from '@/modules/marketplace/presentation/ui/MarketItemList.tsx';
import { useMarketplace } from '@/modules/marketplace/presentation/hooks/useMarketplace.ts';

//TODO: take the filter from zustand store
export const MarketplacePage = () => {
  const { data, isLoading, isError, error } = useMarketplace();

  if (isLoading) return <p>Loading...</p>;
  if (isError) return <p>Error : {error.message}</p>;

  return (
    <div className='flex flex-col bg-background'>
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <MarketItemList items={data} filter={''} />
      </div>
    </div>
  );
};
export default MarketplacePage;
