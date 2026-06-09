import { KeyRound, AlertTriangle, CheckCircle2, Clock3 } from 'lucide-react';
import { Badge, Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import type {
  Credential,
  CredentialKind,
} from '@/modules/features/secret/domain/entities/credential.model.ts';

interface CredentialsGridProps {
  credentials: Credential[];
}

const KIND_LABELS: Record<CredentialKind, string> = {
  SSH_KEY: 'SSH KEY',
  TOKEN: 'TOKEN',
  SECRET_TEXT: 'SECRET TEXT',
  LOGIN: 'LOGIN',
};

const getKindBadgeClass = (kind: CredentialKind) => {
  if (kind === 'SSH_KEY') return 'bg-blue-100 text-blue-700 border-blue-200';
  if (kind === 'TOKEN') return 'bg-emerald-100 text-emerald-700 border-emerald-200';
  if (kind === 'SECRET_TEXT') return 'bg-purple-100 text-purple-700 border-purple-200';
  return 'bg-amber-100 text-amber-700 border-amber-200';
};

const HealthBadge = ({ credential }: { credential: Credential }) => {
  if (credential.health === 'warning') {
    return (
      <div className='flex items-center gap-2 text-xs'>
        <AlertTriangle className='size-3.5 text-red-500' />
        <span className='text-red-500'>Expires in {credential.expiresInDays ?? 0} days</span>
      </div>
    );
  }

  if (credential.health === 'healthy') {
    return (
      <div className='flex items-center gap-2 text-xs'>
        <CheckCircle2 className='size-3.5 text-emerald-500' />
        <span className='text-emerald-500'>Healthy</span>
      </div>
    );
  }

  return (
    <div className='flex items-center gap-2 text-xs'>
      <Clock3 className='size-3.5 text-muted-foreground' />
      <span className='text-muted-foreground'>No usage in 30 days</span>
    </div>
  );
};

export const SecretGrid = ({ credentials }: CredentialsGridProps) => {
  return (
    <div className='w-full h-full overflow-y-auto'>
      <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-3'>
        {credentials.map(credential => (
          <Card key={credential.id} className='rounded-2xl border-border/60 flex flex-col'>
            <CardHeader className='pb-3'>
              <CardTitle className='flex items-start justify-between gap-2'>
                <div className='flex items-center gap-2 flex-1 min-w-0'>
                  <div className='flex size-8 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
                    <KeyRound className='size-3.5 text-primary' />
                  </div>
                  <span className='truncate text-base'>{credential.name}</span>
                </div>
                <Badge
                  variant='outline'
                  className={`${getKindBadgeClass(credential.kind)} text-xs shrink-0`}
                >
                  {KIND_LABELS[credential.kind]}
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className='space-y-3 flex-1 flex flex-col justify-between'>
              <div className='space-y-2 text-sm'>
                <p className='font-mono text-xs text-muted-foreground truncate'>
                  ID: {credential.externalId}
                </p>
                <HealthBadge credential={credential} />
                <p className='text-muted-foreground text-xs'>{credential.lastUsageLabel}</p>
                <p className='text-muted-foreground text-xs'>
                  Created: {credential.createdAtLabel}
                </p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
};
