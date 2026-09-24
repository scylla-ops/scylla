<script lang="ts">
  import { Button, DialogFooter } from '@shadcn';
  import { ScyllaDialog } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { dismissRelease, unseenRelease } from '../whats-new.svelte.ts';
  import { layoutMessages } from './layout.messages.ts';

  const release = $derived(unseenRelease());
</script>

{#if release}
  <ScyllaDialog
    open
    onOpenChange={open => {
      if (!open) dismissRelease();
    }}
    class="sm:max-w-lg"
    title={t(layoutMessages.whatsNew(release.version))}
    description={t(layoutMessages.whatsNewDescription)}
  >
    <ul class="flex flex-col gap-4 py-2">
      {#each release.highlights as highlight (highlight.id)}
        <li class="flex items-start gap-3">
          <span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10">
            <highlight.icon class="size-4 text-primary" />
          </span>
          <div class="min-w-0">
            <p class="text-sm font-semibold text-foreground">{t(highlight.title)}</p>
            <p class="text-sm text-muted-foreground">{t(highlight.description)}</p>
          </div>
        </li>
      {/each}
    </ul>

    <DialogFooter>
      <Button onclick={dismissRelease}>{t(layoutMessages.gotIt)}</Button>
    </DialogFooter>
  </ScyllaDialog>
{/if}
