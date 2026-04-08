import { useMatches, Link } from 'react-router-dom';
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
import { useContextStore } from '@/modules/shared/presentation/stores/useContext';

export const ScyllaBreadcrumbs = () => {
  const matches = useMatches();

  const org = useContextStore(state => state.organization?.name);
  const proj = useContextStore(state => state.project?.name);
  const pipelineName = useContextStore(state => state.pipeline?.name);

  const params: BreadcrumbParams = {
    projectName: proj || undefined,
    organizationName: org || undefined,
    pipelineName: pipelineName || undefined,
  };

  const crumbs = matches
    .filter(match => (match.handle as RouteHandle | undefined)?.breadcrumb)
    .map(match => {
      const handle = match.handle as RouteHandle;
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
          const words = crumb.label.split(' ');

          return (
            <React.Fragment key={crumb.path}>
              <BreadcrumbItem>
                {isLast ? (
                  <BreadcrumbPage className='text-slate-900 font-semibold text-sm px-2 py-1 rounded-md bg-slate-50 flex gap-1'>
                    {words.map((word, i) => {
                      const lastIndex = words.length - 1;
                      const isLastWord = words.length > 1 && i === lastIndex;

                      return (
                        <span key={i} className={isLastWord ? 'text-primary' : 'whitespace-nowrap'}>
                          {word}
                        </span>
                      );
                    })}
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
