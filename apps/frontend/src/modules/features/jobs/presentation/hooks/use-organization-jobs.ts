import { useQuery } from '@tanstack/react-query';
import { useJobsDomain } from '@/modules/features/jobs/presentation/hooks/use-jobs-domain.ts';
import { ORGANIZATION_JOBS_QUERY_KEY } from '@/modules/features/jobs/presentation/hooks/jobs.query-keys.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import {
  isActiveStatus,
  summarizeJobs,
} from '@/modules/features/jobs/domain/structs/jobs-summary.struct.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

/**
 * How many recent runs the organization feed reads.
 *
 * `ListOrganizationJobs` paginates and does not aggregate, so every figure the
 * dashboard shows is computed over this window — see {@link JobsSummary}.
 */
export const ORGANIZATION_JOBS_WINDOW: PaginationParams = { page: 1, pageSize: 100 };

/**
 * The organization's recent runs, plus the outcome mix over that window.
 *
 * Part of the module's public API: the dashboard shows organization-wide run
 * activity, which is a jobs query and belongs here.
 *
 * Polls while something is still running — the same rule the per-pipeline views
 * use — and goes quiet once the window holds only finished jobs.
 */
export const useOrganizationJobs = (organizationId: string | null, enabled = true) => {
  const { jobsRepository } = useJobsDomain();

  const { data, isLoading, isError } = useQuery({
    queryKey: ORGANIZATION_JOBS_QUERY_KEY(organizationId, ORGANIZATION_JOBS_WINDOW),
    queryFn: async () =>
      (
        await jobsRepository.getByOrganizationId(organizationId!, ORGANIZATION_JOBS_WINDOW)
      ).unwrap(),
    enabled: enabled && !!organizationId,
    staleTime: 10_000,
    refetchInterval: query =>
      (query.state.data?.items ?? []).some(job => isActiveStatus(job.status)) ? 5_000 : false,
  });

  const jobs: JobEntity[] = data?.items ?? [];

  return {
    jobs,
    summary: summarizeJobs(jobs),
    totalCount: data?.pagination.totalCount ?? 0,
    /** True when the feed is a window over a larger history, so figures are partial. */
    isPartialWindow: (data?.pagination.totalCount ?? 0) > jobs.length,
    isLoading,
    isError,
  };
};
