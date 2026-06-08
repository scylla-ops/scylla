import { FeatureHeader } from '@shared/presentation/ui';

interface CredentialsHeaderProps {
  activeCount: number;
  onAddCredential?: () => void;
}

export const CredentialsHeader = ({ activeCount, onAddCredential }: CredentialsHeaderProps) => {
  return (
    <FeatureHeader
      onBack={() => {}}
      count={activeCount}
      label={'Credential'}
      pluralLabel={'Credentials'}
      onNew={onAddCredential}
      newLabel={'New credential'}
    />
  );
};
