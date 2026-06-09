import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  CapabilitiesContext,
  DEFAULT_CAPABILITIES,
  type Capabilities,
} from '@core/presentation/contexts/capabilities.context.ts';
import { ServerConfigDataSource } from '@core/infrastructure/grpc/server-config.data-source.ts';
import { CoreModule } from '@core/di/core.module.ts';

const dataSource = new ServerConfigDataSource(CoreModule.data.grpcTransport);

/// Fetches the server capabilities once at boot and exposes them via context.
/// Falls back to all-off while loading or on error, so SaaS-only UI stays hidden
/// unless the server explicitly advertises it.
export const CapabilitiesProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { data } = useQuery({
    queryKey: ['server-config'],
    queryFn: () => dataSource.getServerConfig(),
    staleTime: Infinity,
    retry: false,
  });

  const capabilities: Capabilities = data
    ? {
        signupEnabled: data.signupEnabled,
        invitationsEnabled: data.invitationsEnabled,
        oauthGithubEnabled: data.oauthGithubEnabled,
        meteringEnabled: data.meteringEnabled,
        mailEnabled: data.mailEnabled,
        agentOrgScopeEnabled: data.agentOrgScopeEnabled,
      }
    : DEFAULT_CAPABILITIES;

  return (
    <CapabilitiesContext.Provider value={capabilities}>{children}</CapabilitiesContext.Provider>
  );
};
