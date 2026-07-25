import { FeatureHeader } from '@shared/presentation/ui';
import { Trans } from '@lingui/react/macro';

type FeatureHeaderProps = Parameters<typeof FeatureHeader>[0];

export type RolesHeaderProps = {
  count: number;
  onNew: () => void;
} & Pick<
  FeatureHeaderProps,
  'selectedCount' | 'allSelected' | 'onSelectAll' | 'onClearSelection' | 'onDeleteSelection'
>;

export const RolesHeader = ({ count, onNew, ...selection }: RolesHeaderProps) => {
  return (
    <FeatureHeader
      count={count}
      label={'Role'}
      pluralLabel={'Roles'}
      newLabel={<Trans>Create role</Trans>}
      onNew={onNew}
      {...selection}
    />
  );
};
