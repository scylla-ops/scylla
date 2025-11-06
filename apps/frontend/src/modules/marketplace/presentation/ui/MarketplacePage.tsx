import { MarketItemList } from '@/modules/marketplace/presentation/ui/MarketItemList.tsx';
import { Input } from '@/modules/core/presentation/ui/shadcn';
import { useState } from 'react';
import { useMarketplace } from '@/modules/marketplace/presentation/hooks/useMarketplace.ts';

//TODO: for the topbar, make it reusable for each module of the app
export const MarketplacePage = () => {
  const [filter, setFilter] = useState('');
  const { data, isLoading, isError, error } = useMarketplace();

  if (isLoading) return <p>Loading...</p>;
  if (isError) return <p>Error : {error.message}</p>;

  return (
    <div className='flex flex-col bg-background h-screen'>
      <div className='flex flex-row w-full justify-between bg-gray-100 border-b-2 items-center p-2'>
        <Input
          value={filter}
          onChange={e => setFilter(e.target.value)}
          placeholder='🔍 Search'
          className='w-[20%] bg-background'
        />
        <p>Account: Example account</p>
        <p>Connected in: Example organisation</p>
      </div>
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <MarketItemList items={data} filter={filter} />
      </div>
    </div>
  );
};
export default MarketplacePage;
