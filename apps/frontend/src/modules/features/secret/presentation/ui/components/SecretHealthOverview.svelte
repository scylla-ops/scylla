<script lang="ts">
  import ActivityIcon from '@lucide/svelte/icons/activity';
  import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import { Button, Card, CardContent, CardHeader, CardTitle } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretMessages } from '../secret.messages.ts';

  interface Props {
    warningCount: number;
  }

  let { warningCount }: Props = $props();
</script>

<!--
  Not mounted by any page: `Secret.page.svelte` renders the header and the list
  only, and the figures below are placeholders rather than data. Ported as it
  stood rather than rewritten — see the module's AGENTS.md.
-->
<div class="grid gap-4 md:grid-cols-3">
  <Card class="gap-2 rounded-2xl border-blue-200/40 bg-blue-500/5 py-3">
    <CardHeader class="pb-1">
      <CardTitle class="flex items-center gap-2 text-sm md:text-base">
        <ShieldCheckIcon class="size-4 text-primary" />
        {t(secretMessages.rotationPolicy)}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-1">
      <p class="text-xs text-muted-foreground">
        {t(secretMessages.rotationOverdue(warningCount))}
      </p>
      <Button variant="outline" class="h-8 w-full rounded-xl text-xs">
        {t(secretMessages.reviewPolicy)}
      </Button>
    </CardContent>
  </Card>

  <Card class="gap-2 rounded-2xl border-emerald-200/40 bg-emerald-500/5 py-3">
    <CardHeader class="pb-1">
      <CardTitle class="flex items-center gap-2 text-sm md:text-base">
        <ActivityIcon class="size-4 text-emerald-500" />
        {t(secretMessages.vaultHealth)}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-1">
      <div>
        <p class="text-2xl font-semibold tracking-tight">99.9%</p>
        <p class="text-xs text-muted-foreground">{t(secretMessages.uptimeStatus)}</p>
      </div>
      <p class="text-xs text-muted-foreground">{t(secretMessages.lastSync)}</p>
    </CardContent>
  </Card>

  <Card class="gap-2 rounded-2xl border-primary/20 bg-primary/10 py-3">
    <CardHeader class="pb-1">
      <CardTitle class="flex items-center gap-2 text-sm md:text-base">
        <ClipboardListIcon class="size-4 text-primary" />
        {t(secretMessages.auditLogging)}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-1">
      <div class="space-y-1 text-xs">
        <div class="flex items-center justify-between">
          <span class="text-muted-foreground">{t(secretMessages.accessAttempts)}</span>
          <span class="font-semibold text-foreground">1,402</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-muted-foreground">{t(secretMessages.unauthorizedAttempts)}</span>
          <span class="font-semibold text-destructive">0</span>
        </div>
      </div>
      <Button class="h-8 w-full rounded-xl text-xs">{t(secretMessages.viewAuditLogs)}</Button>
    </CardContent>
  </Card>
</div>
