<script lang="ts">
  import { navigateTo } from '@platform/context';
  import { createMutation } from '@platform/query';
  import { FormDialog, type FormValues } from '@shared/presentation/ui';
  import {
    activeLocale,
    t,
  } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { slugifyOrgName } from '@shared/utils/slug.ts';
  import { organizationMutations } from '../../organization.queries.ts';
  import { createOrganizationItems } from '../../utils/create-organization-form-items.ts';
  import { organizationMessages } from '../organization.messages.ts';

  interface Props {
    open: boolean;
    setOpen: (open: boolean) => void;
    hideCancel?: boolean;
  }

  let { open, setOpen, hideCancel = false }: Props = $props();

  const createOrganization = createMutation(() => organizationMutations.create());

  // The items resolve their labels with the Lingui macro at call time; reading
  // the active locale is what rebuilds them when it changes.
  const items = $derived((activeLocale(), createOrganizationItems()));

  const handleSubmit = ({ name, description }: FormValues<'name' | 'description'>) => {
    if (!name.trim()) return;

    createOrganization.mutate(
      { name, description: description.trim() || undefined },
      {
        // The mutation already makes the new organization the active one; this
        // is the part that only the caller knows — where to land afterwards.
        onSuccess: () => {
          setOpen(false);
          navigateTo(`/${slugifyOrgName(name)}/projects`);
        },
      },
    );
  };
</script>

<FormDialog
  {open}
  onOpenChange={setOpen}
  title={t(organizationMessages.createTitle)}
  description={t(organizationMessages.createDescription)}
  {items}
  isPending={createOrganization.isPending}
  submitLabel={t(organizationMessages.createSubmit)}
  onSubmit={handleSubmit}
  {hideCancel}
/>
