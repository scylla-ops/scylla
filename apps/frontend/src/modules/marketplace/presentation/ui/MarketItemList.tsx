//TODO: move that into domain
import MarketItemCard from '@/modules/marketplace/presentation/ui/MarketItemCard.tsx';
import type { MarketItem } from '@/modules/marketplace/domain/models/MarketItem.ts';

export type MarketItemListProps = {
  items: MarketItem[] | undefined;
  filter: string;
};

export const MarketItemList = ({ items, filter }: MarketItemListProps) => {
  return (
    <div className={'flex flex-row flex-wrap h-fit gap-4'}>
      {items?.map((item, index) => {
        if (item.title.includes(filter) || item.provider.includes(filter))
          return (
            <MarketItemCard
              key={index}
              className='flex-1 min-w-[300px] max-w-[400px] h-50'
              provider={item.provider}
              title={item.title}
              descrption={item.descrption}
            />
          );
      })}
    </div>
  );
};
