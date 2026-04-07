import { useMatches, Link, useParams } from 'react-router-dom';
import type { BreadcrumbParams, RouteHandle } from '@core/presentation/models/RouteHandle.ts';
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

export const ScyllaBreadcrumbs = () => {
  const matches = useMatches();
  const params = useParams();

  // TODO: instead of params of the url, use context
  const crumbs = matches
    .filter(match => (match.handle as RouteHandle | undefined)?.breadcrumb)
    .map(match => {
      const handle = match.handle as RouteHandle;
      const label =
        typeof handle.breadcrumb === 'function'
          ? handle.breadcrumb(params as BreadcrumbParams)
          : handle.breadcrumb;
      return {
        label,
        path: match.pathname,
      };
    });

  if (crumbs.length === 0) return null;

  return (
    <Breadcrumb>
      <BreadcrumbList className='gap-2'>
        {crumbs.map((crumb, index) => {
          const isLast = index === crumbs.length - 1;

          return (
            <React.Fragment key={crumb.path}>
              <BreadcrumbItem>
                {isLast ? (
                  <BreadcrumbPage className='text-slate-900 font-semibold text-sm px-2 py-1 rounded-md bg-slate-50'>
                    {crumb.label}
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
