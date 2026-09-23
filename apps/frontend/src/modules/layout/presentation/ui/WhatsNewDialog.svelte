<script lang="ts">
  import {
    Button,
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { dismissRelease, unseenRelease } from '../whats-new.svelte.ts';
  import { layoutMessages } from './layout.messages.ts';

  const release = $derived(unseenRelease());
</script>

{#if release}
  <Dialog
    open
    onOpenChange={open => {
      if (!open) dismissRelease();
    }}
  >
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>{t(layoutMessages.whatsNew(release.version))}</DialogTitle>
        <DialogDescription>{t(layoutMessages.whatsNewDescription)}</DialogDescription>
      </DialogHeader>

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
    </DialogContent>
  </Dialog>
{/if}
