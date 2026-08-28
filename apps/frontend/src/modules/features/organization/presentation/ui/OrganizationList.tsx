import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { type ComponentType, type ReactNode, useCallback } from 'react';
import { useState } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { ContextItem } from '@/modules/layout/presentation/ui/context-selector/ContextItem.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { Building2, Pencil, Trash, Users } from 'lucide-react';
import { IconButton } from '@shared/presentation/ui';
import { EditOrganizationDialog } from '@/modules/features/organization/presentation/ui/EditOrganizationDialog.tsx';
import { useDeleteOrganization } from '@/modules/features/organization/presentation/hooks/use-delete-organization.ts';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { Trans } from '@lingui/react/macro';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { Can } from '@/modules/features/permission/presentation/ui/authorization/Can.tsx';
import { useNavigate } from 'react-router-dom';

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
      void navigate(`/${slugifyOrgName(otherOrganization.name)}/dashboard`);
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
          className='group rounded-md transition-colors hover:bg-accent/70'
          key={idValue(organisation.organizationId)}
          onSelect={() => {
            setOrganization(idValue(organisation.organizationId), organisation.name);
            void navigate(`/${slugifyOrgName(organisation.name)}/dashboard`);
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
              <Can
                permission={Permission.LIST_ORGANIZATION_MEMBERS}
                target={{ organizationId: idValue(organisation.organizationId) }}
              >
                <IconButton
                  icon={Users}
                  tooltip={<Trans>Members</Trans>}
                  onClick={e => {
                    e.stopPropagation();
                    // The members page reads the organization from the context
                    // store, so looking at another org's members means moving
                    // to it — the row's own click does the same thing.
                    setOrganization(idValue(organisation.organizationId), organisation.name);
                    void navigate(`/${slugifyOrgName(organisation.name)}/members`);
                  }}
                  className='h-7 w-7'
                  iconClassName='h-3.5 w-3.5'
                />
              </Can>
              <Can
                permission={Permission.UPDATE_ORGANIZATION}
                target={{ organizationId: idValue(organisation.organizationId) }}
              >
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
              </Can>
              <Can
                permission={Permission.DELETE_ORGANIZATION}
                target={{ organizationId: idValue(organisation.organizationId) }}
              >
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
              </Can>
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
