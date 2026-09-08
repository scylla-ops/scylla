import { Activity, ClipboardList, ShieldCheck } from 'lucide-react';
import { Button, Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Plural, Trans } from '@lingui/react/macro';

interface CredentialsHealthOverviewProps {
  warningCount: number;
}

export const SecretHealthOverview = ({ warningCount }: CredentialsHealthOverviewProps) => {
  return (
    <div className='grid gap-4 md:grid-cols-3'>
      <Card className='rounded-2xl border-blue-200/40 bg-blue-500/5 py-3 gap-2'>
        <CardHeader className='pb-1'>
          <CardTitle className='flex items-center gap-2 text-sm md:text-base'>
            <ShieldCheck className='size-4 text-primary' />
            <Trans>Rotation Policy</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <p className='text-xs text-muted-foreground'>
            <Plural
              value={warningCount}
              one='# credential overdue for rotation based on your enterprise policy (90 days).'
              other='# credentials overdue for rotation based on your enterprise policy (90 days).'
            />
          </p>
          <Button variant='outline' className='rounded-xl w-full h-8 text-xs'>
            <Trans>Review Policy</Trans>
          </Button>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-emerald-200/40 bg-emerald-500/5 py-3 gap-2'>
        <CardHeader className='pb-1'>
          <CardTitle className='flex items-center gap-2 text-sm md:text-base'>
            <Activity className='size-4 text-emerald-500' />
            <Trans>Vault Health</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <div>
            <p className='text-2xl font-semibold tracking-tight'>99.9%</p>
            <p className='text-xs text-muted-foreground'>
              <Trans>Uptime status</Trans>
            </p>
          </div>
          <p className='text-xs text-muted-foreground'>
            <Trans>Last sync: 2 min ago</Trans>
          </p>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-primary/20 bg-primary/10 py-3 gap-2'>
        <CardHeader className='pb-1'>
          <CardTitle className='flex items-center gap-2 text-sm md:text-base'>
            <ClipboardList className='size-4 text-primary' />
            <Trans>Audit Logging</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <div className='space-y-1 text-xs'>
            <div className='flex items-center justify-between'>
              <span className='text-muted-foreground'>
                <Trans>Access attempts (24h)</Trans>
              </span>
              <span className='font-semibold text-foreground'>1,402</span>
            </div>
            <div className='flex items-center justify-between'>
              <span className='text-muted-foreground'>
                <Trans>Unauthorized attempts</Trans>
              </span>
              <span className='font-semibold text-destructive'>0</span>
            </div>
          </div>
          <Button className='w-full rounded-xl h-8 text-xs'>
            <Trans>View Audit Logs</Trans>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
};
