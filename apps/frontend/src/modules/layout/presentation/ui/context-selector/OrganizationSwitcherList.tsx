import { useCallback, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Building2, Pencil, Trash, Users } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Can, Permission } from '@platform/authz';
import { useContextStore, useScyllaNavigate } from '@platform/context';
import { organizationMutations, organizationQueries } from '@/modules/features/organization';
import { EditOrganizationModal } from './EditOrganizationModal.tsx';
import { ContextItem } from '@shared/presentation/ui/layout/ContextItem.tsx';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { IconButton } from '@shared/presentation/ui';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { DropdownMenuItem } from '@/modules/shared/presentation/ui/shadcn/dropdown-menu.tsx';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/**
 * The organization switcher's list, and shell furniture rather than feature UI.
 *
 * It used to live in `organization` and receive its row wrapper as a component
 * prop, so the sidebar could make each row a `DropdownMenuItem`. That seam is
 * exactly what a Svelte component cannot be given: Radix's menu item provides
 * roving focus and `onSelect` through React context, which reaches React
 * children only. So the rendering moved here, next to the dropdown it belongs
 * to, and `organization` keeps only its data — the queries below come from its
 * barrel.
 *
 * `organization`'s own Svelte `OrganizationList` renders the same list as plain
 * blocks for the user settings panel. The two meet again in Phase 6, when the
 * sidebar becomes Svelte and this file goes.
 */
export const OrganizationSwitcherList = () => {
  const { data: organizations } = useQuery(organizationQueries.mine());
  const setOrganization = useContextStore(state => state.setOrganization);
  const currentOrganizationId = useContextStore(state => state.organization.id);
  const { navigate } = useScyllaNavigate();
  const deleteOrganization = useMutation(organizationMutations.remove());

  const [editOrg, setEditOrg] = useState<{ id: string; name: string; description?: string } | null>(
    null,
  );
  const [deleteOrgId, setDeleteOrgId] = useState<string | null>(null);

  const onDeleteOrganization = useCallback(async () => {
    if (!deleteOrgId) return;

    await deleteOrganization.mutateAsync(deleteOrgId);
    setDeleteOrgId(null);

    if (deleteOrgId !== currentOrganizationId) return;

    const otherOrganization = organizations?.find(org => org.id !== deleteOrgId);
    setOrganization(otherOrganization?.id ?? null, otherOrganization?.name ?? null);
    if (otherOrganization) {
      navigate(`/${slugifyOrgName(otherOrganization.name)}/dashboard`);
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
          <DropdownMenuItem key={i} className='group'>
            <div className='flex items-center gap-3 px-1 py-1'>
              <Skeleton className='h-8 w-8 rounded-md' />
              <Skeleton className='h-4 w-24' />
            </div>
          </DropdownMenuItem>
        ))}
      </>
    );

  return (
    <>
      {organizations.map(organisation => (
        <DropdownMenuItem
          className='group rounded-md transition-colors hover:bg-accent/70'
          key={organisation.id}
          onSelect={() => {
            setOrganization(organisation.id, organisation.name);
            navigate(`/${slugifyOrgName(organisation.name)}/dashboard`);
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
                target={{ organizationId: organisation.id }}
              >
                <IconButton
                  icon={Users}
                  tooltip={<Trans>Members</Trans>}
                  onClick={e => {
                    e.stopPropagation();
                    // The members page reads the organization from the context
                    // store, so looking at another org's members means moving
                    // to it — the row's own click does the same thing.
                    setOrganization(organisation.id, organisation.name);
                    navigate(`/${slugifyOrgName(organisation.name)}/members`);
                  }}
                  className='h-7 w-7'
                  iconClassName='h-3.5 w-3.5'
                />
              </Can>
              <Can
                permission={Permission.UPDATE_ORGANIZATION}
                target={{ organizationId: organisation.id }}
              >
                <IconButton
                  icon={Pencil}
                  tooltip={<Trans>Edit</Trans>}
                  onClick={e => {
                    e.stopPropagation();
                    setEditOrg({
                      id: organisation.id,
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
                target={{ organizationId: organisation.id }}
              >
                <IconButton
                  icon={Trash}
                  tooltip={<Trans>Delete</Trans>}
                  onClick={e => {
                    e.stopPropagation();
                    setDeleteOrgId(organisation.id);
                  }}
                  className='h-7 w-7 hover:text-destructive hover:bg-destructive-subtle'
                  iconClassName='h-3.5 w-3.5'
                />
              </Can>
            </div>
          </div>
        </DropdownMenuItem>
      ))}

      {/* The dialog itself is `organization`'s, and already Svelte: its props
          are plain values, so the island is the whole adapter. */}
      {editOrg && (
        <EditOrganizationModal organization={editOrg} onClose={() => setEditOrg(null)} />
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
