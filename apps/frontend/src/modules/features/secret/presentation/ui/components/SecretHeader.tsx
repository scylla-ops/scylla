import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';

interface CredentialsHeaderProps {
  activeCount: number;
  onAddSecret?: () => void;
  projectId: string;
}

export const SecretHeader = ({ activeCount, onAddSecret, projectId }: CredentialsHeaderProps) => {
  const { clearSelection, selectedIds } = useSelection('secrets');
  const deleteSecret = useDeleteSecret(projectId);

  const handleDelete = () => {
    selectedIds.forEach(id => deleteSecret.mutate(id));
  };
  return (
    <FeatureHeader
      count={activeCount}
      label={'Secret'}
      pluralLabel={'Secrets'}
      onNew={onAddSecret}
      newLabel={'New secret'}
      selectedCount={selectedIds.length}
      onDeleteSelection={handleDelete}
      onClearSelection={clearSelection}
    />
  );
};
