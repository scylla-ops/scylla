import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { HardDriveIcon } from 'lucide-react';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import { AgentsRemoteDataSourceImpl } from '@/modules/features/agents/infrastructure/data/agents-remote.data-source.ts';
import { grpcTransport } from '@platform/grpc/index.ts';
import { DefaultAgentsRepository } from '@/modules/features/agents/infrastructure/repository/default-agents.repository.ts';

const dataSource = new AgentsRemoteDataSourceImpl(grpcTransport);
const repository = new DefaultAgentsRepository(dataSource);

export const AgentsModule = {
  id: 'agents',
  domain: {
    /** Repository interface — the module's data surface. */
    agentsRepository: repository,
  },
  routes: [
    {
      mount: 'organization',
      path: 'agents',
      breadcrumb: () => ({ label: msg`Agents` }),
      children: [
        {
          mount: 'organization',
          index: true,
          permission: Permission.LIST_AGENTS,
          lazy: async () => ({
            Component: (await import('./presentation/ui/Agents.page.tsx')).AgentsPage,
          }),
        },
        {
          mount: 'organization',
          path: ':agentId',
          permission: Permission.LIST_AGENTS,
          breadcrumb: () => ({ label: msg`Agent details` }),
          lazy: async () => ({
            Component: (await import('./presentation/ui/AgentDetails.page.tsx')).AgentDetailsPage,
          }),
        },
      ],
    },
  ],
  nav: [
    {
      section: 'organization',
      title: msg`Agents`,
      url: 'agents',
      icon: HardDriveIcon,
      permission: Permission.LIST_AGENTS,
      order: 40,
    },
  ],
} satisfies ScyllaModule;
