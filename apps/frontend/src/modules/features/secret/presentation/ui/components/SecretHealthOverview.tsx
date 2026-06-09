import { Activity, ClipboardList, ShieldCheck } from 'lucide-react';
import { Button, Card, CardContent, CardHeader, CardTitle } from '@shadcn';

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
            Rotation Policy
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <p className='text-xs text-muted-foreground'>
            {warningCount} credential{warningCount > 1 ? 's' : ''} overdue for rotation based on
            your enterprise policy (90 days).
          </p>
          <Button variant='outline' className='rounded-xl w-full h-8 text-xs'>
            Review Policy
          </Button>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-emerald-200/40 bg-emerald-500/5 py-3 gap-2'>
        <CardHeader className='pb-1'>
          <CardTitle className='flex items-center gap-2 text-sm md:text-base'>
            <Activity className='size-4 text-emerald-500' />
            Vault Health
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <div>
            <p className='text-2xl font-semibold tracking-tight'>99.9%</p>
            <p className='text-xs text-muted-foreground'>Uptime status</p>
          </div>
          <p className='text-xs text-muted-foreground'>Last sync: 2 min ago</p>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-primary/20 bg-primary/10 py-3 gap-2'>
        <CardHeader className='pb-1'>
          <CardTitle className='flex items-center gap-2 text-sm md:text-base'>
            <ClipboardList className='size-4 text-primary' />
            Audit Logging
          </CardTitle>
        </CardHeader>
        <CardContent className='space-y-1'>
          <div className='space-y-1 text-xs'>
            <div className='flex items-center justify-between'>
              <span className='text-muted-foreground'>Access attempts (24h)</span>
              <span className='font-semibold text-foreground'>1,402</span>
            </div>
            <div className='flex items-center justify-between'>
              <span className='text-muted-foreground'>Unauthorized attempts</span>
              <span className='font-semibold text-destructive'>0</span>
            </div>
          </div>
          <Button className='w-full rounded-xl h-8 text-xs'>View Audit Logs</Button>
        </CardContent>
      </Card>
    </div>
  );
};
