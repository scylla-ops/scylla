import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import WorkflowIcon from '@lucide/svelte/icons/workflow';
import { Permission } from '@platform/authz';
import { GrpcProjectRemoteDataSource } from '@/modules/features/project/infrastructure/data/grpc-project-remote.data-source.ts';
import { DefaultProjectRepository } from '@/modules/features/project/infrastructure/repository/default-project.repository.ts';
import { grpcTransport } from '@platform/grpc';

const projectRemoteDataSource = new GrpcProjectRemoteDataSource(grpcTransport);
const projectRepository = new DefaultProjectRepository(projectRemoteDataSource);

export const ProjectModule = {
  id: 'project',
  domain: {
    projectRepository: projectRepository,
  },
  routes: {
    organization: [
      {
        path: 'projects',
        // Reading the organization is the real gate.
        permission: Permission.READ_ORGANIZATION,
        breadcrumb: () => ({ label: msg`Projects` }),
        page: () => import('./presentation/ui/Project.page.svelte'),
        nav: { section: 'organization', title: msg`Projects`, icon: WorkflowIcon, order: 20 },
      },
    ],
  },
} satisfies ScyllaModule;
