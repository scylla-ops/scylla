<script lang="ts">
  import { i18n } from '@lingui/core';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { scyllaNavigate } from '@platform/context';
  import {
    Badge,
    Button,
    buttonVariants,
    Card,
    CardContent,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@shadcn-svelte';
  import { cn } from '@shared/presentation/utils';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AppEntity } from '../../../domain/entities/app.entity.ts';
  import { appsMessages } from '../apps.messages.ts';

  interface Props {
    app: AppEntity;
    onRequestDelete: (id: string) => void;
    /** False disables both delete affordances — the backend checks `DeleteApp`. */
    canDelete?: boolean;
  }

  let { app, onRequestDelete, canDelete = true }: Props = $props();

  const copyId = async () => {
    await navigator.clipboard.writeText(app.id);
    toast.success(i18n._(ToastMessages.APP_ID_COPIED));
  };
</script>

<Card
  class="cursor-pointer gap-0 py-0 transition-colors hover:bg-accent/50"
  onclick={() => scyllaNavigate.goToSubRoute(app.id)}
>
  <CardContent class="p-4">
    <div class="flex items-start justify-between gap-2">
      <div class="flex min-w-0 items-center gap-3">
        <span
          class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-success/30 bg-success/10"
        >
          <KeyRoundIcon class="h-4 w-4 text-success" />
        </span>
        <div class="min-w-0">
          <p class="truncate font-semibold">{app.name}</p>
          <p class="truncate font-mono text-xs text-muted-foreground">{app.id}</p>
        </div>
      </div>
      <DropdownMenu>
        <!--
          The trigger *is* the button, rather than wrapping one through the
          `child` snippet — the same choice `IconButton` makes. bits-ui already
          renders a `<button>` and merges what it is given, so dressing it with
          `buttonVariants` yields the one element React's `asChild` produced,
          and `mergeProps` chains our `onclick` with the menu's instead of one
          silently replacing the other.
        -->
        <DropdownMenuTrigger
          class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'h-7 w-7 shrink-0')}
          onclick={(event: MouseEvent) => event.stopPropagation()}
        >
          <MoreHorizontalIcon class="h-4 w-4" />
          <!-- Icon-only: without this the trigger has no accessible name. -->
          <span class="sr-only">{t(appsMessages.appActions)}</span>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" onclick={(event: MouseEvent) => event.stopPropagation()}>
          <DropdownMenuItem onSelect={() => void copyId()}>
            <CopyIcon class="mr-2 h-4 w-4" />
            {t(appsMessages.copyId)}
          </DropdownMenuItem>
          <DropdownMenuItem
            variant="destructive"
            disabled={!canDelete}
            onSelect={() => onRequestDelete(app.id)}
          >
            <TrashIcon class="mr-2 h-4 w-4" />
            {t(appsMessages.delete)}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    {#if !app.isActive}
      <Badge variant="secondary" class="mt-3">{t(appsMessages.inactive)}</Badge>
    {/if}
  </CardContent>

  <div class="flex items-center justify-between border-t border-dashed px-4 py-2.5">
    <span class="text-xs text-muted-foreground">
      {t(appsMessages.created)}
      {formatDate(app.createdAt)}
    </span>
    <Button
      variant="ghost"
      size="icon"
      disabled={!canDelete}
      class="h-7 w-7 text-muted-foreground hover:text-destructive"
      onclick={event => {
        event.stopPropagation();
        onRequestDelete(app.id);
      }}
    >
      <TrashIcon class="h-4 w-4" />
      <!-- Icon-only: without this the control has no accessible name, and no
           test can reach it except through its CSS classes. -->
      <span class="sr-only">{t(appsMessages.deleteApp)}</span>
    </Button>
  </div>
</Card>
