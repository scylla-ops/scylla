import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { ShoppingCartIcon } from 'lucide-react';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import { DefaultMarketplaceRepository } from '@/modules/features/marketplace/infrastructure/repository/default-marketplace.repository.ts';

const marketPlaceRepository: MarketplaceRepository = new DefaultMarketplaceRepository();

export const MarketplaceModule = {
  id: 'marketplace',
  domain: {
    /** Repository interface — the module's data surface. */
    marketplaceRepository: marketPlaceRepository,
  },
  routes: [
    {
      mount: 'organization',
      path: 'marketplace',
      permission: Permission.LIST_APPS_BY_ORGANIZATION,
      lazy: async () => ({
        Component: (await import('./presentation/ui/Marketplace.page.tsx')).MarketplacePage,
      }),
    },
  ],
  nav: [
    {
      section: 'organization',
      title: msg`Marketplace`,
      url: 'marketplace',
      icon: ShoppingCartIcon,
      permission: Permission.LIST_APPS_BY_ORGANIZATION,
      order: 50,
    },
  ],
} satisfies ScyllaModule;
