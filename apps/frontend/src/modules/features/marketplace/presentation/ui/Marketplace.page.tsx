import { Trans } from '@lingui/react/macro';
import { Card } from '@shadcn';

export const MarketplacePage = () => {
  return (
    <div className='flex items-center justify-center w-full h-full bg-background'>
      <Card className='p-8 text-center'>
        <p className='text-lg text-muted-foreground'>
          <Trans>This feature will be available soon</Trans>
        </p>
      </Card>
    </div>
  );
};
export default MarketplacePage;
