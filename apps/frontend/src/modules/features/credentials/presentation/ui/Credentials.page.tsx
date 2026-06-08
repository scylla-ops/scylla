import { useCredentials } from '@/modules/features/credentials/presentation/hooks/use-credentials.ts';
import {
  CredentialsHeader,
  CredentialsHealthOverview,
  CredentialsFilters,
  CredentialsList,
  CredentialsGrid,
  CredentialsPagination,
} from '@/modules/features/credentials/presentation/ui/components/index.ts';

export const CredentialsPage = () => {
  const {
    credentials,
    healthCount,
    kindFilter,
    page,
    search,
    setKindFilter,
    setPage,
    setSearch,
    setView,
    totalItems,
    totalPages,
    view,
  } = useCredentials();

  return (
    <div className='flex flex-col gap-4 w-full min-h-full p-2'>
      <CredentialsHeader
        activeCount={healthCount.healthy + healthCount.warning + healthCount.warning}
        onAddCredential={() => {}}
      />

      <CredentialsHealthOverview warningCount={healthCount.warning} />

      <CredentialsFilters
        search={search}
        onSearchChange={setSearch}
        kindFilter={kindFilter}
        onKindFilterChange={setKindFilter}
        view={view}
        onViewChange={setView}
      />

      <div className='flex-1 min-h-0 overflow-hidden'>
        {view === 'list' ? (
          <CredentialsList credentials={credentials} />
        ) : (
          <CredentialsGrid credentials={credentials} />
        )}
      </div>

      <CredentialsPagination
        page={page}
        totalPages={totalPages}
        totalItems={totalItems}
        itemsPerPage={5}
        onPageChange={setPage}
      />
    </div>
  );
};

export default CredentialsPage;
