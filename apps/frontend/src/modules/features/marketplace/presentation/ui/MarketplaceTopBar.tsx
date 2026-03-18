import { Input } from '@/modules/shared/presentation/ui/shadcn/input.tsx';

import { useFilterStore } from '@/modules/features/marketplace/presentation/stores/useFilter.ts';

//todo: data-sources filter in zustand data-sources
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
