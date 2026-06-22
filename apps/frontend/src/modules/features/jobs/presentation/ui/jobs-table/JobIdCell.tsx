import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import { Trans } from '@lingui/react/macro';
import { CopyableText } from '@shared/presentation/ui/data-display/CopyableText.tsx';

export function JobIdCell({ job }: { job: Job }) {
  return <CopyableText value={job.id} truncate={12} copyLabel={<Trans>Copy ID</Trans>} />;
}

export default JobIdCell;
