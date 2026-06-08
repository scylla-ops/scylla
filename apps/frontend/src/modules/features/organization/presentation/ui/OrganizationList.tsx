import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { type ComponentType, type ReactNode, useCallback } from 'react';
import { useState } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { ContextItem } from '@/modules/layout/presentation/ui/context-selector/ContextItem.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { Building2, Pencil, Trash } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { IconButton } from '@shared/presentation/ui';
import { EditOrganizationDialog } from '@/modules/features/organization/presentation/ui/EditOrganizationDialog.tsx';
import { useDeleteOrganization } from '@/modules/features/organization/presentation/hooks/use-delete-organization.ts';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { Trans } from '@lingui/react/macro';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { idValue } from '@core/infrastructure/grpc/wrappers.ts';

interface OrganizationListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void; className?: string }>;
}

export const OrganizationList = ({ Wrapper }: OrganizationListProps) => {
  const { organizations } = useOrganizations();
  const setOrganization = useContextStore(state => state.setOrganization);
  const currentOrganizationId = useContextStore(state => state.organization.id);
  const navigate = useNavigate();
  const deleteOrganization = useDeleteOrganization();

  const [editOrg, setEditOrg] = useState<{ id: string; name: string; description?: string } | null>(
    null,
  );
  const [deleteOrgId, setDeleteOrgId] = useState<string | null>(null);

  const onDeleteOrganization = useCallback(async () => {
    if (!deleteOrgId) return;

    await deleteOrganization.mutateAsync(deleteOrgId);
    setDeleteOrgId(null);

    if (deleteOrgId !== currentOrganizationId) return;

    const otherOrganization = organizations?.find(
      org => idValue(org.organizationId) !== deleteOrgId,
    );
    setOrganization(
      otherOrganization ? idValue(otherOrganization.organizationId) : null,
      otherOrganization?.name ?? null,
    );
    if (otherOrganization) {
      navigate(`/${slugifyOrgName(otherOrganization.name)}/projects`);
    }
  }, [
    deleteOrgId,
    deleteOrganization,
    currentOrganizationId,
    organizations,
    setOrganization,
    navigate,
  ]);

  if (!organizations)
    return (
      <>
        {Array.from({ length: 3 }).map((_, i) => (
          <Wrapper key={i} className='group'>
            <div className='flex items-center gap-3 px-1 py-1'>
              <Skeleton className='h-8 w-8 rounded-md' />
              <Skeleton className='h-4 w-24' />
            </div>
          </Wrapper>
        ))}
      </>
    );

  return (
    <>
      {organizations.map(organisation => (
        <Wrapper
          className={'group hover:bg-slate-50 transition-colors'}
          key={idValue(organisation.organizationId)}
          onSelect={() => {
            setOrganization(idValue(organisation.organizationId), organisation.name);
            navigate(`/${slugifyOrgName(organisation.name)}/projects`);
          }}
        >
          <div className='flex items-center w-full'>
            <div className='flex-1 min-w-0'>
              <ContextItem
                name={organisation.name}
                description={organisation.description}
                icon={Building2}
              />
            </div>
            <div className='flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity'>
              <IconButton
                icon={Pencil}
                tooltip={<Trans>Edit</Trans>}
                onClick={e => {
                  e.stopPropagation();
                  setEditOrg({
                    id: idValue(organisation.organizationId),
                    name: organisation.name,
                    description: organisation.description,
                  });
                }}
                className='h-7 w-7'
                iconClassName='h-3.5 w-3.5'
              />
              <IconButton
                icon={Trash}
                tooltip={<Trans>Delete</Trans>}
                onClick={e => {
                  e.stopPropagation();
                  setDeleteOrgId(idValue(organisation.organizationId));
                }}
                className='h-7 w-7 hover:text-destructive hover:bg-destructive/10'
                iconClassName='h-3.5 w-3.5'
              />
            </div>
          </div>
        </Wrapper>
      ))}

      {editOrg && (
        <EditOrganizationDialog
          open={!!editOrg}
          setOpen={open => {
            if (!open) setEditOrg(null);
          }}
          organization={editOrg}
        />
      )}

      <ConfirmOperationAlertDialog
        open={!!deleteOrgId}
        onOpenChange={open => {
          if (!open) setDeleteOrgId(null);
        }}
        onContinue={onDeleteOrganization}
      />
    </>
  );
};
