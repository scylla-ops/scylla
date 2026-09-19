import { LazySvelteIsland } from '@shared/presentation/ui/svelte/LazySvelteIsland.tsx';
import { loadEditOrganizationDialog } from '@/modules/features/organization';

/** The edit-organization dialog, mounted from the React shell. */
export const EditOrganizationModal = ({
  organization,
  onClose,
}: {
  organization: { id: string; name: string; description?: string };
  onClose: () => void;
}) => (
  <LazySvelteIsland
    load={loadEditOrganizationDialog}
    props={{
      open: true,
      setOpen: (open: boolean) => {
        if (!open) onClose();
      },
      organization,
    }}
  />
);
