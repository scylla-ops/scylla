import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { WorkflowIcon } from 'lucide-react';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import { GrpcProjectRemoteDataSource } from '@/modules/features/project/infrastructure/data/grpc-project-remote.data-source.ts';
import { DefaultProjectRepository } from '@/modules/features/project/infrastructure/repository/default-project.repository.ts';
import { grpcTransport } from '@platform/grpc/index.ts';

const projectRemoteDataSource = new GrpcProjectRemoteDataSource(grpcTransport);
const projectRepository = new DefaultProjectRepository(projectRemoteDataSource);

export const ProjectModule = {
  id: 'project',
  domain: {
    /** Repository interface — the module's data surface. */
    projectRepository: projectRepository,
  },
  routes: [
    {
      mount: 'projects',
      index: true,
      // Reading the organization is the real gate.
      permission: Permission.READ_ORGANIZATION,
      lazy: async () => ({
        Component: (await import('./presentation/ui/ProjectPage.tsx')).ProjectPage,
      }),
    },
  ],
  nav: [
    {
      section: 'organization',
      title: msg`Projects`,
      url: 'projects',
      icon: WorkflowIcon,
      permission: Permission.READ_ORGANIZATION,
      order: 20,
    },
  ],
} satisfies ScyllaModule;
