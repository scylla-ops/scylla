<script lang="ts">
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { membershipMessages } from '../membership.messages.ts';

  interface Props {
    roles: { roleId: string; name: string }[];
    disabled: boolean;
    onSelect: (roleId: string) => void;
  }

  let { roles, disabled, onSelect }: Props = $props();
</script>

<!--
  Grants one more role to someone already listed, in one click.

  A select rather than a dialog because adding a role to an existing member is a
  correction, not a decision worth a modal — and it stays on the row it is about.
  Renders nothing once there is nothing left to add.

  The height override carries the `data-[size=sm]` prefix on purpose: the
  trigger's own `data-[size=sm]:h-8` outranks a plain `h-6` on specificity, so an
  unprefixed override silently loses and the control stands a row taller than the
  chips beside it.
-->
{#if roles.length > 0}
  <Select
    type="single"
    value=""
    {disabled}
    onValueChange={value => {
      if (value) onSelect(value);
    }}
  >
    <SelectTrigger
      size="sm"
      class="w-auto gap-1 border-dashed px-2 text-xs text-muted-foreground data-[size=sm]:h-6"
    >
      <SelectValue placeholder={t(membershipMessages.addRolePlaceholder)} />
    </SelectTrigger>
    <SelectContent>
      {#each roles as role (role.roleId)}
        <SelectItem value={role.roleId} label={role.name}>{role.name}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
{/if}
