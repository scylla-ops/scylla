import { useMatches, Link, useParams } from 'react-router-dom';
import type {
  BreadcrumbParams,
  RouteHandle,
} from '@core/presentation/structs/route-handle.struct.ts';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@shadcn/breadcrumb.tsx';
import { ChevronRight } from 'lucide-react';
import React from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

export const ScyllaBreadcrumbs = () => {
  const matches = useMatches();

  const org = useContextStore(state => state.organization?.name);
  const proj = useContextStore(state => state.project?.name);
  const pipelineName = useContextStore(state => state.pipeline?.name);

  const { userId } = useParams();

  const params: BreadcrumbParams = {
    projectName: proj || undefined,
    organizationName: org || undefined,
    pipelineName: pipelineName || undefined,
    userId: userId || undefined,
  };

  const crumbs = matches
    .filter(match => (match.handle as RouteHandle | undefined)?.breadcrumb)
    .map(match => ({
      ...(match.handle as Required<RouteHandle>).breadcrumb(params),
      path: match.pathname,
    }));

  if (crumbs.length === 0) return null;

  return (
    <Breadcrumb>
      <BreadcrumbList className='gap-2'>
        {crumbs.map((crumb, index) => {
          const isLast = index === crumbs.length - 1;

          const content = (
            <>
              <span className='whitespace-nowrap'>{crumb.label}</span>
              {crumb.highlight && <span className='text-primary'>#{crumb.highlight}</span>}
              {crumb.detail && <span className='whitespace-nowrap'>- {crumb.detail}</span>}
            </>
          );

          return (
            <React.Fragment key={crumb.path}>
              <BreadcrumbItem>
                {isLast ? (
                  <BreadcrumbPage className='font-semibold text-sm px-2 py-1 rounded-md bg-muted text-foreground flex gap-1 items-center'>
                    {content}
                  </BreadcrumbPage>
                ) : (
                  <BreadcrumbLink
                    asChild
                    className='text-muted-foreground hover:text-primary font-medium text-sm px-2 py-1 rounded-md hover:bg-primary/10 transition-all duration-200'
                  >
                    <Link to={crumb.path} className='flex gap-1 items-center'>
                      {content}
                    </Link>
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
              {!isLast && (
                <BreadcrumbSeparator>
                  <ChevronRight className='w-4 h-4 text-muted-foreground' />
                </BreadcrumbSeparator>
              )}
            </React.Fragment>
          );
        })}
      </BreadcrumbList>
    </Breadcrumb>
  );
};
