// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/features/login/presentation/ui/Login.page.tsx';
import { MarketplacePage } from '@/modules/features/marketplace/presentation/ui/Marketplace.page.tsx';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { AuthGuard } from '@core/presentation/ui/router/Auth.guard.tsx';
import ProjectPage from '@/modules/features/project/presentation/ui/ProjectPage.tsx';
import type { BreadcrumbParams } from '@core/presentation/models/route-handle.model.ts';
import { ContextCleanerWrapper } from './ContextCleaner.wrapper.tsx';
import { JobsPage } from '@/modules/features/jobs/presentation/ui/Jobs.page.tsx';
import { UserAdminPage } from '@/modules/features/user/presentation/ui/admin/UserAdmin.page.tsx';
import UserSettingsPage from '@/modules/features/user/presentation/ui/settings/UserSettings.page.tsx';
import { DashboardPipelinePage } from '@/modules/features/pipeline/presentation/ui/dashboard/DashboardPipeline.page.tsx';
import { PipelineCreationPage } from '@/modules/features/pipeline/presentation/ui/editor/PipelineCreation.page.tsx';
import { PipelineUpdatePage } from '@/modules/features/pipeline/presentation/ui/editor/PipelineUpdate.page.tsx';
import { OrganizationSyncWrapper } from './OrganizationSync.wrapper.tsx';
import { OrganizationRedirectWrapper } from './OrganizationRedirect.wrapper.tsx';
import { SecretPage } from '@/modules/features/secret/presentation/ui/Secret.page.tsx';
import { AgentsPage } from '@/modules/features/agents/presentation/ui/Agents.page.tsx';
import { AgentDetailsPage } from '@/modules/features/agents/presentation/ui/AgentDetails.page.tsx';
import { RolesPage } from '@/modules/features/permission/presentation/ui/Roles.page.tsx';

//TODO: put each navigations part in a separate file, (module ?)
export const CoreRouter = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },

  {
    element: <AuthGuard />,
    children: [
      {
        element: <Layout />,
        children: [
          {
            index: true,
            element: <OrganizationRedirectWrapper />,
          },
          {
            path: '/:organizationSlug',
            element: <OrganizationSyncWrapper />,
            children: [
              {
                path: 'projects',
                handle: {
                  breadcrumb: () => 'Projects',
                },
                children: [
                  {
                    index: true,
                    element: <ProjectPage />,
                  },
                  {
                    element: <ContextCleanerWrapper />,
                    //TODO: here add loader: <ContextLoader/> used to fetch the project and pipeline names from ids for the breadcrumbs
                    path: ':projectId',
                    handle: {
                      breadcrumb: (params: BreadcrumbParams) => `Project #${params.projectName}`,
                    },
                    children: [
                      {
                        index: true,
                        element: <DashboardPipelinePage />,
                      },
                      {
                        path: 'secrets',
                        element: <SecretPage />,
                        handle: {
                          breadcrumb: () => 'Secrets',
                        },
                      },
                      {
                        path: 'create',
                        element: <PipelineCreationPage />,
                        handle: {
                          breadcrumb: () => `Create`,
                        },
                      },
                      {
                        path: 'edit/:pipelineId',
                        element: <PipelineUpdatePage />,
                        handle: {
                          breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                            `Pipeline #${pipelineName} - Edit`,
                        },
                      },
                      {
                        path: 'pipelines/:pipelineId/jobs',
                        element: <JobsPage />,
                        handle: {
                          breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                            `Pipeline #${pipelineName} - Jobs`,
                        },
                      },
                    ],
                  },
                ],
              },
              {
                path: 'marketplace',
                element: <MarketplacePage />,
              },
              {
                path: 'agents',
                children: [
                  {
                    index: true,
                    element: <AgentsPage />,
                    handle: {
                      breadcrumb: () => 'Agents',
                    },
                  },
                  {
                    path: ':agentId',
                    element: <AgentDetailsPage />,
                    handle: {
                      breadcrumb: () => 'Agent details',
                    },
                  },
                ],
              },
              {
                path: 'roles',
                element: <RolesPage />,
              },
              {
                path: 'users-admin',
                element: <UserAdminPage />,
              },
              {
                path: 'users',
                children: [
                  {
                    index: true,
                    element: <UserAdminPage />,
                  },
                  {
                    path: ':userId',
                    element: <UserSettingsPage />,
                  },
                  {
                    path: 'me',
                    element: <UserSettingsPage />,
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to='/login' replace />,
  },
]);
