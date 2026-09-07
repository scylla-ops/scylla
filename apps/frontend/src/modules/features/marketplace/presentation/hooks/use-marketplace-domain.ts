import { useModuleDomain } from '@platform/di';
import type { MarketplaceModule } from '../../marketplace.module.ts';

/** Typed access to the marketplace module's use cases. */
export const useMarketplaceDomain = () => useModuleDomain<typeof MarketplaceModule.domain>('marketplace');
