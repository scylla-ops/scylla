import { Input } from '@shadcn/input.tsx';

import { useFilterStore } from '@/modules/marketplace/presentation/stores/useFilter.ts';

//todo: dataSources filter in zustand dataSources
export const MarketplaceTopBar = () => {
  const filter = useFilterStore(state => state.filter);
  const setFilter = useFilterStore(state => state.setFilter);

  return (
    <Input
      value={filter}
      onChange={e => setFilter(e.target.value)}
      placeholder='🔍 Search'
      className='w-[20%] bg-background'
    />
  );
};
