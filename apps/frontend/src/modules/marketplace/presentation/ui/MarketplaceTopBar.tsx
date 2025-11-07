import { Input } from '@shadcn/input.tsx';
import { useState } from 'react';

//todo: store filter in zustand store
export const MarketplaceTopBar = () => {
  const [filter, setFilter] = useState('');

  return (
    <Input
      value={filter}
      onChange={e => setFilter(e.target.value)}
      placeholder='🔍 Search'
      className='w-[20%] bg-background'
    />
  );
};
