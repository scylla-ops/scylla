import { MarketItemList } from '@/modules/features/marketplace/presentation/ui/MarketItemList.tsx';
import { useMarketplace } from '@/modules/features/marketplace/presentation/hooks/use-marketplace.ts';
import { useFilterStore } from '@/modules/features/marketplace/presentation/stores/use-filter.store.ts';
import { ErrorState } from '@/modules/shared/presentation/ui/ErrorState.tsx';

export const MarketplacePage = () => {
  const { data, isLoading, isError, error } = useMarketplace();
  const filter = useFilterStore(state => state.filter);

  if (isLoading) return <></>;

  if (isError) return <ErrorState message={error.message} />;

  return (
    <div className='flex flex-col bg-background'>
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <MarketItemList items={data} filter={filter} />
      </div>
    </div>
  );
};
export default MarketplacePage;
