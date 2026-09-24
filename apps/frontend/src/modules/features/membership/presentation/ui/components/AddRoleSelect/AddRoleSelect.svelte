<script lang="ts">
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    roles: { roleId: string; name: string }[];
    disabled: boolean;
    onSelect: (roleId: string) => void;
  }

  let { roles, disabled, onSelect }: Props = $props();
</script>

<!--
  Adds one role to a listed member. Renders nothing when no role is left.
  The height override keeps the `data-[size=sm]` prefix to win on specificity.
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
