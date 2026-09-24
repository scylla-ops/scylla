import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import ShoppingCartIcon from '@lucide/svelte/icons/shopping-cart';
import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import { DefaultMarketplaceRepository } from '@/modules/features/marketplace/infrastructure/repository/default-marketplace.repository.ts';

const marketPlaceRepository: MarketplaceRepository = new DefaultMarketplaceRepository();

export const MarketplaceModule = {
  id: 'marketplace',
  domain: {
    /** Repository interface — the module's data surface. */
    marketplaceRepository: marketPlaceRepository,
  },
  routes: {
    organization: [
      {
        path: 'marketplace',
        page: () => import('./presentation/ui/Marketplace.page.svelte'),
        nav: {
          section: 'organization',
          title: msg`Marketplace`,
          icon: ShoppingCartIcon,
          order: 50,
        },
      },
    ],
  },
} satisfies ScyllaModule;
