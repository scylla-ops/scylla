import { Input } from '@/modules/shared/presentation/ui/shadcn/input.tsx';

import { useFilterStore } from '@/modules/features/marketplace/presentation/stores/useFilter.ts';
import { useLingui } from '@lingui/react/macro';

//todo: data-sources filter in zustand data-sources
export const MarketplaceTopBar = () => {
  const { t } = useLingui();
  const filter = useFilterStore(state => state.filter);
  const setFilter = useFilterStore(state => state.setFilter);

  return (
    <Input
      value={filter}
      onChange={e => setFilter(e.target.value)}
      placeholder={t`Search`}
      className='w-[20%] bg-background'
    />
  );
};
