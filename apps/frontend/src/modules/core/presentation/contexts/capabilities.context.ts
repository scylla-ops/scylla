import { createContext } from 'react';

/// Server feature flags mirrored from the backend's cargo features. Drives
/// conditional UI (register, GitHub login, invitations, …).
export interface Capabilities {
  signupEnabled: boolean;
  invitationsEnabled: boolean;
  oauthGithubEnabled: boolean;
  meteringEnabled: boolean;
  mailEnabled: boolean;
  agentOrgScopeEnabled: boolean;
}

/// Safe default before discovery resolves / if the call fails: everything off
/// (PaaS-like). Hiding a SaaS button is the safe failure mode.
export const DEFAULT_CAPABILITIES: Capabilities = {
  signupEnabled: false,
  invitationsEnabled: false,
  oauthGithubEnabled: false,
  meteringEnabled: false,
  mailEnabled: false,
  agentOrgScopeEnabled: false,
};

export const CapabilitiesContext = createContext<Capabilities>(DEFAULT_CAPABILITIES);
