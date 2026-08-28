import type { ReactNode } from 'react';
import { Folder, Workflow, ChevronRight } from 'lucide-react';
import { Trans, useLingui } from '@lingui/react/macro';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Badge } from '@shadcn/badge.tsx';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { Separator } from '@shadcn/separator.tsx';
import { FeatureHeader } from '@shared/presentation/ui/layout/FeatureHeader.tsx';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { cn } from '@shared/presentation/utils';
import { getRelativeTime } from '@shared/utils/date-utils.ts';
import { useOrgOverview } from '@/modules/features/dashboard/presentation/hooks/use-org-overview.ts';
import { AgentOutcomesChart } from '@/modules/features/dashboard/presentation/ui/AgentOutcomesChart.tsx';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { Can } from '@/modules/features/permission/presentation/ui/authorization/Can.tsx';

const StatCard = ({
  icon,
  label,
  value,
  loading,
}: {
  icon: ReactNode;
  label: string;
  value: number;
  loading: boolean;
}) => (
  <Card className='py-5'>
    <CardContent className='px-5 pb-0'>
      <div className='flex items-center gap-3'>
        <div className='rounded-lg bg-primary/10 p-2 shrink-0'>{icon}</div>
        <div>
          {loading ? (
            <Skeleton className='h-7 w-10 mb-1' />
          ) : (
            <p className='text-2xl font-bold leading-none'>{value}</p>
          )}
          <p className='text-xs text-muted-foreground mt-1'>{label}</p>
        </div>
      </div>
    </CardContent>
  </Card>
);

export const DashboardPage = () => {
  const { t } = useLingui();
  const {
    projects,
    projectsLoading,
    projectsError,
    allPipelines,
    pipelinesLoading,
    canOpenProject,
  } = useOrgOverview();
  const navigate = useScyllaNavigate();

  if (projectsError) return <ErrorState message={t`Unable to load dashboard`} />;

  const sortedPipelines = [...allPipelines].sort(
    (a, b) => a.projectName.localeCompare(b.projectName) || a.name.localeCompare(b.name),
  );

  // A project the user may see listed but not open: leave the row inert rather
  // than send them to a permission-denied panel.
  const openableProject = (projectId: string): ProjectEntity | undefined => {
    const project = projects.find(p => p.id === projectId);
    return project && canOpenProject(project.id) ? project : undefined;
  };

  return (
    // The page is a fixed frame: header, stats and chart keep their own height
    // (`shrink-0`) and the two lists share what is left, each scrolling inside
    // its own box. The lists carry a `min-h` floor so they stay usable instead
    // of being squeezed to a sliver; if a viewport is too short to honour even
    // that, the frame itself scrolls rather than collapsing them out of sight.
    // `pr-3` keeps right-aligned content ("See all", the table's last column)
    // off the scroll track; it is unconditional so the layout does not shift
    // depending on whether a scrollbar happens to be showing.
    <div className='flex h-full min-h-0 w-full flex-col gap-4 overflow-y-auto pr-3'>
      <FeatureHeader label={t`Dashboard`} />

      <div className='grid shrink-0 grid-cols-2 gap-4 max-w-sm'>
        <StatCard
          icon={<Folder className='h-4 w-4 text-primary' />}
          label={t`Projects`}
          value={projects.length}
          loading={projectsLoading}
        />
        <StatCard
          icon={<Workflow className='h-4 w-4 text-primary' />}
          label={t`Pipelines`}
          value={allPipelines.length}
          loading={pipelinesLoading || projectsLoading}
        />
      </div>

      <Separator className='shrink-0' />

      <section className='flex flex-1 flex-col gap-3'>
        <div className='flex shrink-0 items-center justify-between'>
          <h2 className='text-base font-semibold'>
            <Trans>Projects</Trans>
          </h2>
          <button
            type='button'
            onClick={() => navigate.goToOrgRoute('/projects')}
            className='flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground'
          >
            <Trans>See all</Trans>
            <ChevronRight className='h-3.5 w-3.5' />
          </button>
        </div>

        <div className='min-h-[8rem] flex-1 overflow-y-auto pr-2'>
          {projectsLoading ? (
            <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3'>
              {Array.from({ length: 4 }).map((_, i) => (
                <Skeleton key={i} className='h-20 w-full rounded-xl' />
              ))}
            </div>
          ) : projects.length === 0 ? (
            <p className='text-sm text-muted-foreground'>
              <Trans>No projects yet.</Trans>
            </p>
          ) : (
            <div className='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3'>
              {projects.map(project => {
                const openable = canOpenProject(project.id);
                return (
                  <Card
                    key={project.id}
                    onClick={openable ? () => navigate.goToProject(project) : undefined}
                    title={
                      openable ? undefined : t`You don't have access to this project's pipelines`
                    }
                    className={cn(
                      'py-4 gap-2 transition-all',
                      openable
                        ? 'cursor-pointer hover:shadow-md hover:border-primary/50 active:scale-[0.98]'
                        : 'opacity-60',
                    )}
                  >
                    <CardHeader className='px-4 pb-0'>
                      <div className='flex items-center gap-2'>
                        <div className='rounded-md bg-primary/10 p-1.5 shrink-0'>
                          <Folder className='h-3.5 w-3.5 text-primary' />
                        </div>
                        <CardTitle className='text-sm font-semibold truncate' title={project.name}>
                          {project.name}
                        </CardTitle>
                      </div>
                    </CardHeader>
                    <CardContent className='px-4 pb-0'>
                      <p className='text-xs text-muted-foreground truncate'>
                        {project.description || (
                          <span className='italic'>
                            <Trans>No description</Trans>
                          </span>
                        )}
                      </p>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          )}
        </div>
      </section>

      <Separator className='shrink-0' />

      <section className='flex flex-1 flex-col gap-3'>
        <h2 className='shrink-0 text-base font-semibold'>
          <Trans>All Pipelines</Trans>
        </h2>

        <div className='min-h-[10rem] flex-1 overflow-y-auto pr-2'>
          {projectsLoading || pipelinesLoading ? (
            <div className='flex flex-col gap-2'>
              {Array.from({ length: 5 }).map((_, i) => (
                <Skeleton key={i} className='h-12 w-full rounded-xl' />
              ))}
            </div>
          ) : sortedPipelines.length === 0 ? (
            <p className='text-sm text-muted-foreground'>
              <Trans>No pipelines yet.</Trans>
            </p>
          ) : (
            <div className='rounded-xl border overflow-hidden'>
              <table className='w-full text-sm'>
                {/* Sticky inside the section's scroll box — needs an opaque
                    background so rows don't show through as they pass under. */}
                <thead className='sticky top-0 z-10 border-b bg-muted'>
                  <tr>
                    <th className='px-4 py-3 text-left font-medium text-muted-foreground'>
                      <Trans>Name</Trans>
                    </th>
                    <th className='hidden px-4 py-3 text-left font-medium text-muted-foreground sm:table-cell'>
                      <Trans>Project</Trans>
                    </th>
                    <th className='hidden px-4 py-3 text-left font-medium text-muted-foreground md:table-cell'>
                      <Trans>Nodes</Trans>
                    </th>
                    <th className='hidden px-4 py-3 text-left font-medium text-muted-foreground md:table-cell'>
                      <Trans>Updated</Trans>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {sortedPipelines.map((pipeline, idx) => {
                    const project = openableProject(pipeline.projectId);
                    return (
                      <tr
                        key={pipeline.id}
                        onClick={() => project && navigate.goToProject(project)}
                        className={cn(
                          'transition-colors hover:bg-muted/40',
                          project && 'cursor-pointer',
                          idx > 0 && 'border-t',
                        )}
                      >
                        <td className='px-4 py-3 font-medium'>{pipeline.name}</td>
                        <td className='hidden px-4 py-3 sm:table-cell'>
                          <Badge variant='secondary'>{pipeline.projectName}</Badge>
                        </td>
                        <td className='hidden px-4 py-3 text-muted-foreground md:table-cell'>
                          {pipeline.nodeCount}
                        </td>
                        <td className='hidden px-4 py-3 text-muted-foreground md:table-cell'>
                          {getRelativeTime(pipeline.updatedAt)}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </section>

      {/* The chart reads agent stats — hide it whole rather than render a panel
          that can only fail for a user without agent access. */}
      <Can permission={Permission.LIST_AGENTS}>
        <Separator className='shrink-0' />

        <section className='flex shrink-0 flex-col gap-3'>
          <h2 className='text-base font-semibold'>
            <Trans>Agent Outcomes</Trans>
          </h2>
          <AgentOutcomesChart />
        </section>
      </Can>
    </div>
  );
};
