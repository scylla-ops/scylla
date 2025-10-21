import MarketItemCard, {
  type MarketItemCardProps,
} from '@/modules/marketplace/presentation/ui/MarketItemCard.tsx';
import {
  type MarketItem,
  MarketItemList,
} from '@/modules/marketplace/presentation/ui/MarketItemList.tsx';

export const MarketplacePage = () => {
  const items: MarketItem[] = [
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },

    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
    { provider: 'Corp', title: 'Title', descrption: 'Description' },
  ];

  return (
    <div className={'flex bg-background h-screen p-2'}>
      <MarketItemList items={items} />
    </div>
  );
};
export default MarketplacePage;
