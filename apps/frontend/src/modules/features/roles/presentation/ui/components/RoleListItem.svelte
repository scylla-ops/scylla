<script lang="ts">
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import { Badge, Checkbox } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../domain/entities/role.entity.ts';
  import { scopeLabelOf } from '../../utils/permission-mapping.ts';
  import { rolesMessages } from '../roles.messages.ts';

  interface Props {
    role: RoleEntity;
    memberCount: number;
    active: boolean;
    selected: boolean;
    /** False for builtins, or without the rights. */
    selectable: boolean;
    onOpen: () => void;
    onToggleSelect: () => void;
  }

  let { role, memberCount, active, selected, selectable, onOpen, onToggleSelect }: Props =
    $props();
</script>

<div
  role="button"
  tabindex="0"
  onclick={onOpen}
  onkeydown={event => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onOpen();
    }
  }}
  class={cn(
    'group flex w-full cursor-pointer items-center gap-4 overflow-hidden rounded-xl border p-4 transition-all duration-200',
    active
      ? 'border-primary/40 bg-primary/6 hover:border-primary/40'
      : 'hover:border-slate-300 hover:shadow-sm',
  )}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="flex w-5 items-center justify-center"
    onclick={event => event.stopPropagation()}
  >
    <Checkbox
      checked={selected}
      disabled={!selectable}
      aria-label={role.name}
      onCheckedChange={onToggleSelect}
    />
  </div>

  <div class="mr-4 flex w-full min-w-0 flex-1 items-center gap-3 overflow-hidden">
    <div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10">
      <ShieldCheckIcon class="size-4 text-primary" />
    </div>
    <div class="flex min-w-0 flex-1 flex-col items-start">
      <p class="w-full truncate font-semibold text-foreground">{role.name}</p>
      <p class="w-full truncate text-xs text-muted-foreground">
        {role.description || t(rolesMessages.noDescription)}
      </p>
    </div>
  </div>

  <div class="flex items-center gap-1.5">
    <Badge variant="secondary">{scopeLabelOf(role.scope)}</Badge>
    <Badge variant="outline">{t(rolesMessages.memberCount(memberCount))}</Badge>
  </div>
</div>
