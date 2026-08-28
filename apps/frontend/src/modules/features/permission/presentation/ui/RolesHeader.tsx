import { FeatureHeader } from '@shared/presentation/ui';
import { Trans } from '@lingui/react/macro';

type FeatureHeaderProps = Parameters<typeof FeatureHeader>[0];

export type RolesHeaderProps = {
  count: number;
  /** Omit to hide the "Create role" button (e.g. the user can't manage roles). */
  onNew?: () => void;
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
