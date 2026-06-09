import { useContext } from 'react';
import { CapabilitiesContext } from '@core/presentation/contexts/capabilities.context.ts';

/// Read the server's enabled features anywhere in the tree, e.g.
/// `const { signupEnabled } = useCapabilities();`
export const useCapabilities = () => useContext(CapabilitiesContext);
