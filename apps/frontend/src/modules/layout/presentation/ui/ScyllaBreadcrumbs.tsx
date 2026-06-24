import { useMatches, Link, useParams } from 'react-router-dom';
import type {
  BreadcrumbParams,
  RouteHandleModel,
} from '@core/presentation/models/route-handle.model.ts';
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
    .filter(match => (match.handle as RouteHandleModel | undefined)?.breadcrumb)
    .map(match => {
      const handle = match.handle as RouteHandleModel;
      const label =
        typeof handle.breadcrumb === 'function' ? handle.breadcrumb(params) : handle.breadcrumb;
      return {
        label: String(label),
        path: match.pathname,
      };
    });

  if (crumbs.length === 0) return null;

  return (
    <Breadcrumb>
      <BreadcrumbList className='gap-2'>
        {crumbs.map((crumb, index) => {
          const isLast = index === crumbs.length - 1;

          const firstHashIndex = crumb.label.indexOf('#');
          let beforeId = crumb.label;
          let id = '';
          let afterId = '';

          if (firstHashIndex !== -1) {
            beforeId = crumb.label.substring(0, firstHashIndex);

            const afterHash = crumb.label.substring(firstHashIndex);
            const separatorIndex = afterHash.indexOf(' - ');

            if (separatorIndex !== -1) {
              id = afterHash.substring(0, separatorIndex);
              afterId = afterHash.substring(separatorIndex);
            } else {
              id = afterHash;
            }
          }

          return (
            <React.Fragment key={crumb.path}>
              <BreadcrumbItem>
                {isLast ? (
                  <BreadcrumbPage className='text-slate-900 font-semibold text-sm px-2 py-1 rounded-md bg-slate-50 flex gap-1 items-center'>
                    {id ? (
                      <>
                        <span className='whitespace-nowrap'>{beforeId}</span>
                        <span className='text-primary'>{id}</span>
                        <span className='whitespace-nowrap'>{afterId}</span>
                      </>
                    ) : (
                      <span>{crumb.label}</span>
                    )}
                  </BreadcrumbPage>
                ) : (
                  <BreadcrumbLink
                    asChild
                    className='text-slate-600 hover:text-primary font-medium text-sm px-2 py-1 rounded-md hover:bg-primary/5 transition-all duration-200'
                  >
                    <Link to={crumb.path}>{crumb.label}</Link>
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
              {!isLast && (
                <BreadcrumbSeparator>
                  <ChevronRight className='w-4 h-4 text-slate-400' />
                </BreadcrumbSeparator>
              )}
            </React.Fragment>
          );
        })}
      </BreadcrumbList>
    </Breadcrumb>
  );
};
