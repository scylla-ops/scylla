import { sveltePage, type ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { ShoppingCartIcon } from 'lucide-react';
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
      lazy: async () => ({
        Component: sveltePage(
          (await import('./presentation/ui/Marketplace.page.svelte')).default,
        ),
      }),
    },
  ],
  nav: [
    {
      section: 'organization',
      title: msg`Marketplace`,
      url: 'marketplace',
      icon: ShoppingCartIcon,
      order: 50,
    },
  ],
} satisfies ScyllaModule;
