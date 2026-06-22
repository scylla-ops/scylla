import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';

interface CredentialsHeaderProps {
  activeCount: number;
  secretIds: string[];
  onAddSecret?: () => void;
  projectId: string;
}

export const SecretHeader = ({
  activeCount,
  secretIds,
  onAddSecret,
  projectId,
}: CredentialsHeaderProps) => {
  const deleteSecret = useDeleteSecret(projectId);
  const { headerProps } = useFeatureSelection('secrets', secretIds, {
    deleteItem: id => deleteSecret.mutateAsync(id),
  });

  return (
    <FeatureHeader
      count={activeCount}
      label={'Secret'}
      pluralLabel={'Secrets'}
      onNew={onAddSecret}
      newLabel={'New secret'}
      {...headerProps}
    />
  );
};
