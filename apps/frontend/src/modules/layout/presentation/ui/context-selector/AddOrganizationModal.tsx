import { LazySvelteIsland } from '@shared/presentation/ui/svelte/LazySvelteIsland.tsx';
import { loadAddOrganizationDialog } from '@/modules/features/organization';

/**
 * The create-organization dialog, mounted from the React shell.
 *
 * `organization` is migrated and the dialog is Svelte; every prop it takes is a
 * plain value or a callback, so the island is the whole adapter. Deleted in
 * Phase 6 along with `SvelteIsland` itself.
 */
export const AddOrganizationModal = ({
  open,
  setOpen,
}: {
  open: boolean;
  setOpen: (open: boolean) => void;
}) => <LazySvelteIsland load={loadAddOrganizationDialog} props={{ open, setOpen }} />;
