import {
  Button,
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import { Trans } from '@lingui/react/macro';

export type MarketItemCardProps = {
  provider: string;
  title: string;
  descrption: string;
  className?: string;
};

export const MarketItemCard = ({ provider, title, descrption, className }: MarketItemCardProps) => {
  return (
    <Card className={`${className} bg-card flex flex-col h-fit min-h-[250px] gap-2`}>
      <CardHeader className='flex items-center justify-between'>
        <img src={LogoScylla} alt='logo' className='w-24 h-24' />
        <CardAction>
          <CardDescription>{provider}</CardDescription>
        </CardAction>
      </CardHeader>

      <CardContent className='flex-1'>
        <CardTitle className='text-2xl'>{title}</CardTitle>
        <p>{descrption}</p>
      </CardContent>

      <CardFooter className={'flex justify-end gap-2'}>
        <Button variant={'outline'}><Trans>Learn more</Trans></Button>
        <Button><Trans>Download</Trans></Button>
      </CardFooter>
    </Card>
  );
};

export default MarketItemCard;
