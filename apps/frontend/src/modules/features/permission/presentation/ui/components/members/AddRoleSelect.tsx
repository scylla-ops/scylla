import { useLingui } from '@lingui/react/macro';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shadcn';

interface AddRoleSelectProps {
  roles: { roleId: string; name: string }[];
  disabled: boolean;
  onSelect: (roleId: string) => void;
}

/**
 * Grants one more role to someone already listed, in one click.
 *
 * A select rather than a dialog because adding a role to an existing member is
 * a correction, not a decision worth a modal — and it stays on the row it is
 * about. Renders nothing once there is nothing left to add.
 *
 * The height override carries the `data-[size=sm]` prefix on purpose: the
 * trigger's own `data-[size=sm]:h-8` outranks a plain `h-6` on specificity, so
 * an unprefixed override silently loses and the control stands a row taller
 * than the chips beside it.
 */
export const AddRoleSelect = ({ roles, disabled, onSelect }: AddRoleSelectProps) => {
  const { t } = useLingui();
  if (roles.length === 0) return null;

  return (
    <Select value='' disabled={disabled} onValueChange={onSelect}>
      <SelectTrigger
        size='sm'
        className='w-auto gap-1 border-dashed px-2 text-xs text-muted-foreground data-[size=sm]:h-6'
      >
        <SelectValue placeholder={t`+ role`} />
      </SelectTrigger>
      <SelectContent>
        {roles.map(role => (
          <SelectItem key={role.roleId} value={role.roleId}>
            {role.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

export default AddRoleSelect;
