import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { Trans } from '@lingui/react/macro';
import { CopyableText } from '@shared/presentation/ui/data-display/CopyableText.tsx';

export function JobIdCell({ job }: { job: JobEntity }) {
  return <CopyableText value={job.id} truncate={12} copyLabel={<Trans>Copy ID</Trans>} />;
}

export default JobIdCell;
