import { useEffect, useMemo, useState } from 'react';
import type {
  Credential,
  CredentialHealth,
  CredentialKind,
} from '@/modules/features/credentials/domain/models/credential.model.ts';

const PAGE_SIZE = 5;

const MOCK_CREDENTIALS: Credential[] = [
  {
    id: 'cred-001',
    name: 'production-k8s-ssh',
    externalId: 'k8s_admin_rsa',
    kind: 'SSH_KEY',
    health: 'warning',
    lastUsageLabel: 'Used 10 mins ago',
    createdAtLabel: 'Oct 24, 2023',
    expiresInDays: 12,
  },
  {
    id: 'cred-002',
    name: 'github-ci-pat',
    externalId: 'gh_token_actions',
    kind: 'TOKEN',
    health: 'healthy',
    lastUsageLabel: 'Used 2 hours ago',
    createdAtLabel: 'Nov 02, 2023',
  },
  {
    id: 'cred-003',
    name: 'aws-secret-access-key',
    externalId: 'aws_deploy_key',
    kind: 'SECRET_TEXT',
    health: 'idle',
    lastUsageLabel: 'Last used Dec 01, 2023',
    createdAtLabel: 'Sep 15, 2023',
  },
  {
    id: 'cred-004',
    name: 'docker-hub-login',
    externalId: 'registry_auth',
    kind: 'LOGIN',
    health: 'healthy',
    lastUsageLabel: 'Used 4 mins ago',
    createdAtLabel: 'Dec 20, 2023',
  },
  {
    id: 'cred-005',
    name: 'vault-agent-token',
    externalId: 'vault_runtime_token',
    kind: 'TOKEN',
    health: 'healthy',
    lastUsageLabel: 'Used 1 min ago',
    createdAtLabel: 'Jan 10, 2024',
  },
  {
    id: 'cred-006',
    name: 'helm-repo-login',
    externalId: 'helm_repo_login',
    kind: 'LOGIN',
    health: 'idle',
    lastUsageLabel: 'No usage in 21 days',
    createdAtLabel: 'Jan 30, 2024',
  },
  {
    id: 'cred-007',
    name: 'gitops-signing-key',
    externalId: 'gitops_sign_key',
    kind: 'SSH_KEY',
    health: 'healthy',
    lastUsageLabel: 'Used 3 mins ago',
    createdAtLabel: 'Feb 02, 2024',
  },
  {
    id: 'cred-008',
    name: 'slack-webhook-secret',
    externalId: 'slack_webhook_prod',
    kind: 'SECRET_TEXT',
    health: 'warning',
    lastUsageLabel: 'Used 45 mins ago',
    createdAtLabel: 'Feb 12, 2024',
    expiresInDays: 5,
  },
  {
    id: 'cred-009',
    name: 'artifact-registry-token',
    externalId: 'gcp_artifact_token',
    kind: 'TOKEN',
    health: 'healthy',
    lastUsageLabel: 'Used 6 mins ago',
    createdAtLabel: 'Mar 01, 2024',
  },
  {
    id: 'cred-010',
    name: 'terraform-cloud-login',
    externalId: 'tf_cloud_login',
    kind: 'LOGIN',
    health: 'healthy',
    lastUsageLabel: 'Used 14 mins ago',
    createdAtLabel: 'Mar 18, 2024',
  },
];

export type CredentialsView = 'list' | 'grid';

export const useCredentials = () => {
  const [search, setSearch] = useState('');
  const [kindFilter, setKindFilter] = useState<'ALL' | CredentialKind>('ALL');
  const [view, setView] = useState<CredentialsView>('list');
  const [page, setPage] = useState(1);

  const filteredCredentials = useMemo(() => {
    const normalized = search.trim().toLowerCase();

    return MOCK_CREDENTIALS.filter(credential => {
      const matchesKind = kindFilter === 'ALL' || credential.kind === kindFilter;
      if (!normalized) {
        return matchesKind;
      }

      const matchesSearch =
        credential.name.toLowerCase().includes(normalized) ||
        credential.externalId.toLowerCase().includes(normalized) ||
        credential.kind.toLowerCase().includes(normalized);

      return matchesKind && matchesSearch;
    });
  }, [search, kindFilter]);

  const totalPages = Math.max(1, Math.ceil(filteredCredentials.length / PAGE_SIZE));

  useEffect(() => {
    setPage(currentPage => Math.min(currentPage, totalPages));
  }, [totalPages]);

  const paginatedCredentials = useMemo(() => {
    const start = (page - 1) * PAGE_SIZE;
    return filteredCredentials.slice(start, start + PAGE_SIZE);
  }, [filteredCredentials, page]);

  const healthCount = useMemo(() => {
    return MOCK_CREDENTIALS.reduce<Record<CredentialHealth, number>>(
      (acc, credential) => {
        acc[credential.health] += 1;
        return acc;
      },
      { healthy: 0, warning: 0, idle: 0 },
    );
  }, []);

  return {
    page,
    setPage,
    search,
    setSearch,
    view,
    setView,
    kindFilter,
    setKindFilter,
    credentials: paginatedCredentials,
    totalItems: filteredCredentials.length,
    totalPages,
    healthCount,
  };
};

