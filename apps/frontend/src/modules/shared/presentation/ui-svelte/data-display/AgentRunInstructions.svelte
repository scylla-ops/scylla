<script lang="ts">
  import ContainerIcon from '@lucide/svelte/icons/container';
  import TerminalIcon from '@lucide/svelte/icons/terminal';
  import { CodeSnippet, Tabs, TabsContent, TabsList, TabsTrigger } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { agentRunInstructionsMessages as messages } from './agent-run-instructions.messages.ts';

  interface Props {
    /** The agent's app id (its identity towards the control plane). */
    appId: string;
    /** One-time plaintext secret (creation flow). Omitted → placeholder. */
    secret?: string;
  }

  let { appId, secret }: Props = $props();

  const SECRET_PLACEHOLDER = '<APP_SECRET>';
  const DOCKER_IMAGE = 'godlyjaaaaj/scylla-agent:latest';

  const secretValue = $derived(secret ?? SECRET_PLACEHOLDER);
  // The control plane serves this very page, so the browser's own address bar
  // already holds the value the snippets ask the user to type.
  const controlPlaneUrl = window.location.origin;

  const cargoCommand = $derived(
    [
      'cargo run --release --bin scylla-agent -- \\',
      `  --control-plane-url ${controlPlaneUrl} \\`,
      `  --app-id ${appId} \\`,
      `  --app-secret ${secretValue} \\`,
      '  --workspace-root "$HOME/.scylla/workspaces"',
    ].join('\n'),
  );

  const dockerCommand = $derived(
    [
      `docker run ${DOCKER_IMAGE} \\`,
      `  --control-plane-url ${controlPlaneUrl} \\`,
      `  --app-id ${appId} \\`,
      `  --app-secret ${secretValue}`,
    ].join('\n'),
  );

  const triggerClass =
    'h-auto flex-none gap-1.5 rounded-none border-0 border-b-2 border-transparent bg-transparent ' +
    'px-0 pb-2 text-xs text-muted-foreground shadow-none transition-colors hover:text-foreground ' +
    'data-[state=active]:border-primary data-[state=active]:text-primary ' +
    'data-[state=active]:bg-transparent data-[state=active]:shadow-none';
</script>

<!--
  "How do I start this worker?" — the copy-paste answer, in the two supported
  flavours: `cargo run` from a repo checkout (first — no extra tooling concepts
  beyond Rust itself), or the published Docker image. Shared by the one-time
  secret dialog (shown after the reveal) and the agent detail page (placeholder
  secret).
-->
<div class="w-full space-y-3">
  <Tabs value="cargo" class="w-full">
    <TabsList
      class="h-auto w-full justify-start gap-4 rounded-none border-b border-border bg-transparent p-0"
    >
      <TabsTrigger value="cargo" class={triggerClass}>
        <TerminalIcon class="h-3.5 w-3.5" />
        {t(messages.fromSource)}
      </TabsTrigger>
      <TabsTrigger value="docker" class={triggerClass}>
        <ContainerIcon class="h-3.5 w-3.5" />
        Docker
      </TabsTrigger>
    </TabsList>

    <TabsContent value="cargo" class="mt-2.5 space-y-1.5 focus-visible:outline-none">
      <CodeSnippet
        multiline
        value={cargoCommand}
        copyToast={t(messages.commandCopied)}
        label={cargoLabel}
      />
      <p class="text-xs text-muted-foreground leading-normal">
        {t(messages.requiresRustBefore)}
        <a
          href="https://github.com/scylla-ops/scylla"
          target="_blank"
          rel="noreferrer"
          class="underline underline-offset-2 hover:text-foreground"
        >
          {t(messages.scyllaRepoLink)}
        </a>
        {t(messages.requiresRustAfter)}
      </p>
    </TabsContent>

    <TabsContent value="docker" class="mt-2.5 space-y-1.5 focus-visible:outline-none">
      <CodeSnippet
        multiline
        value={dockerCommand}
        copyToast={t(messages.commandCopied)}
        label={dockerLabel}
      />
      <p class="text-xs text-muted-foreground leading-normal">{t(messages.dockerForeground)}</p>
    </TabsContent>
  </Tabs>

  <p class="text-xs text-muted-foreground leading-normal">{t(messages.replaceUrl)}</p>

  {#if !secret}
    <p
      class="rounded-md border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground leading-normal"
    >
      {t(messages.replaceSecret)}
    </p>
  {/if}
</div>

{#snippet cargoLabel()}{t(messages.runFromCheckout)}{/snippet}
{#snippet dockerLabel()}{t(messages.runWithDocker)}{/snippet}
