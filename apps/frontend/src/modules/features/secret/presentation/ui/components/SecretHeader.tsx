import { FeatureHeader } from '@shared/presentation/ui';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

interface CredentialsHeaderProps {
  activeCount: number;
  onAddSecret?: () => void;
}

export const SecretHeader = ({ activeCount, onAddSecret }: CredentialsHeaderProps) => {
  const { goBack } = useScyllaNavigate();
  return (
    <FeatureHeader
      onBack={goBack}
      count={activeCount}
      label={'Secret'}
      pluralLabel={'Secrets'}
      onNew={onAddSecret}
      newLabel={'New secret'}
    />
  );
};
