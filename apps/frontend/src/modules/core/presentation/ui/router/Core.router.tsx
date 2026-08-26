// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/features/login/presentation/ui/Login.page.tsx';
import { MarketplacePage } from '@/modules/features/marketplace/presentation/ui/Marketplace.page.tsx';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { AuthGuard } from '@core/presentation/ui/router/Auth.guard.tsx';
import ProjectPage from '@/modules/features/project/presentation/ui/ProjectPage.tsx';
import type { BreadcrumbParams } from '@core/presentation/structs/route-handle.struct.ts';
import { ContextCleanerWrapper } from './ContextCleaner.wrapper.tsx';
import { JobsPage } from '@/modules/features/jobs/presentation/ui/Jobs.page.tsx';
import { TriggersPage } from '@/modules/features/triggers/presentation/ui/Triggers.page.tsx';
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
import { RequirePermission } from '@/modules/features/permission/presentation/ui/authorization/RequirePermission.tsx';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';

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
                    element: (
                      // Reading the organization is the real gate: the backend
                      // then returns every project (with LIST_PROJECTS_BY_ORGANIZATION)
                      // or just the ones the caller holds a grant on.
                      <RequirePermission permission={Permission.READ_ORGANIZATION}>
                        <ProjectPage />
                      </RequirePermission>
                    ),
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
                        element: (
                          <RequirePermission permission={Permission.LIST_PIPELINES_BY_PROJECT}>
                            <DashboardPipelinePage />
                          </RequirePermission>
                        ),
                      },
                      {
                        path: 'secrets',
                        element: (
                          <RequirePermission permission={Permission.LIST_SECRETS}>
                            <SecretPage />
                          </RequirePermission>
                        ),
                        handle: {
                          breadcrumb: () => 'Secrets',
                        },
                      },
                      {
                        path: 'create',
                        element: (
                          <RequirePermission permission={Permission.CREATE_PIPELINE}>
                            <PipelineCreationPage />
                          </RequirePermission>
                        ),
                        handle: {
                          breadcrumb: () => `Create`,
                        },
                      },
                      {
                        path: 'edit/:pipelineId',
                        element: (
                          <RequirePermission permission={Permission.UPDATE_PIPELINE}>
                            <PipelineUpdatePage />
                          </RequirePermission>
                        ),
                        handle: {
                          breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                            `Pipeline #${pipelineName} - Edit`,
                        },
                      },
                      {
                        path: 'pipelines/:pipelineId/jobs',
                        element: (
                          <RequirePermission permission={Permission.LIST_JOBS_BY_PIPELINE}>
                            <JobsPage />
                          </RequirePermission>
                        ),
                        handle: {
                          breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                            `Pipeline #${pipelineName} - Jobs`,
                        },
                      },
                      {
                        path: 'pipelines/:pipelineId/triggers',
                        element: (
                          <RequirePermission permission={Permission.MANAGE_TRIGGERS}>
                            <TriggersPage />
                          </RequirePermission>
                        ),
                        handle: {
                          breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                            `Pipeline #${pipelineName} - Triggers`,
                        },
                      },
                    ],
                  },
                ],
              },
              {
                path: 'marketplace',
                element: (
                  <RequirePermission permission={Permission.LIST_APPS_BY_ORGANIZATION}>
                    <MarketplacePage />
                  </RequirePermission>
                ),
              },
              {
                path: 'agents',
                children: [
                  {
                    index: true,
                    element: (
                      <RequirePermission permission={Permission.LIST_AGENTS}>
                        <AgentsPage />
                      </RequirePermission>
                    ),
                    handle: {
                      breadcrumb: () => 'Agents',
                    },
                  },
                  {
                    path: ':agentId',
                    element: (
                      <RequirePermission permission={Permission.LIST_AGENTS}>
                        <AgentDetailsPage />
                      </RequirePermission>
                    ),
                    handle: {
                      breadcrumb: () => 'Agent details',
                    },
                  },
                ],
              },
              {
                path: 'users',
                handle: {
                  breadcrumb: () => 'Users',
                },
                children: [
                  {
                    index: true,
                    element: (
                      <RequirePermission permission={Permission.LIST_USERS}>
                        <UserAdminPage />
                      </RequirePermission>
                    ),
                  },
                  {
                    path: ':userId',
                    element: <UserSettingsPage />,
                    handle: {
                      breadcrumb: ({ userId }: BreadcrumbParams) => `User #${userId} - Detail`,
                    },
                  },
                ],
              },
              {
                path: 'roles',
                element: (
                  <RequirePermission permission={Permission.MANAGE_ROLES}>
                    <RolesPage />
                  </RequirePermission>
                ),
                handle: {
                  breadcrumb: () => 'Roles',
                },
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
