import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import HardDriveIcon from '@lucide/svelte/icons/hard-drive';
import { Permission } from '@platform/authz';
import { AgentsRemoteDataSourceImpl } from '@/modules/features/agents/infrastructure/data/agents-remote.data-source.ts';
import { grpcTransport } from '@platform/grpc';
import { DefaultAgentsRepository } from '@/modules/features/agents/infrastructure/repository/default-agents.repository.ts';

const dataSource = new AgentsRemoteDataSourceImpl(grpcTransport);
const repository = new DefaultAgentsRepository(dataSource);

export const AgentsModule = {
  id: 'agents',
  domain: {
    /** Repository interface — the module's data surface. */
    agentsRepository: repository,
  },
  routes: {
    organization: [
      {
        path: 'agents',
        permission: Permission.LIST_AGENTS,
        breadcrumb: () => ({ label: msg`Agents` }),
        page: () => import('./presentation/ui/Agents.page.svelte'),
        nav: { section: 'organization', title: msg`Agents`, icon: HardDriveIcon, order: 40 },
        children: [
          {
            path: ':agentId',
            permission: Permission.READ_APP,
            breadcrumb: () => ({ label: msg`Agent details` }),
            page: () => import('./presentation/ui/AgentDetails.page.svelte'),
          },
        ],
      },
    ],
  },
} satisfies ScyllaModule;
