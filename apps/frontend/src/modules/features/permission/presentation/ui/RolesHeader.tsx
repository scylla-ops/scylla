import { FeatureHeader } from '@shared/presentation/ui';

export type RolesHeaderProps = {
  count: number;
};

export const RolesHeader = ({ count }: RolesHeaderProps) => {
  return (
    <div>
      <FeatureHeader
        count={count}
        label={'Role'}
        pluralLabel={'Roles'}
        newLabel={'Create Role'}
        onNew={() => {}}
      />
    </div>
  );
};
