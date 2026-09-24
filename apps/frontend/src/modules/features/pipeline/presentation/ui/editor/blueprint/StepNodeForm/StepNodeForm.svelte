<script lang="ts">
  import { StreamLanguage } from '@codemirror/language';
  import { shell as shellLanguage } from '@codemirror/legacy-modes/mode/shell';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import { createQuery } from '@platform/query';
  import { secretQueries } from '@/modules/features/secret';
  import {
    Button,
    DialogFooter,
    Input,
    Label,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { renderCodeMirror } from '@shared/presentation/ui';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { PipelineStep, Shell } from '../../../../../domain/structs/pipeline.struct.ts';
  import type { NodeFormValue } from '../../../../utils/blueprint-converter.ts';
  import { createScriptEditor } from '../../script-editor.svelte.ts';
  import { createStepNodeForm, type EnvRow } from '../step-node-form.state.svelte.ts';
  import { pipelineMessages } from '../../../../pipeline.messages.ts';

  interface Props {
    /** The step being edited; `undefined` means a new one is being defined. */
    editingStep?: PipelineStep;
    /** Scopes the secret picker offered for a `secret` environment entry. */
    projectId?: string;
    onSubmit: (nodeId: string, value: NodeFormValue) => void;
    onCancel: () => void;
  }

  let { editingStep, projectId, onSubmit, onCancel }: Props = $props();

  // Seeded once, deliberately: `StepNodeFormDialog` renders this under
  // `{#key open}`, so a new opening builds a new component rather than
  // re-reading the prop.
  // svelte-ignore state_referenced_locally
  const form = createStepNodeForm(editingStep);

  // A plain options object, shared with the `secret` module — same query key,
  // same cache entry.
  const secretsQuery = createQuery(() => secretQueries.byProject(projectId ?? ''));
  const secrets = $derived(secretsQuery.data ?? []);

  const editor = createScriptEditor({
    value: () => form.script,
    onChange: next => (form.script = next),
    extensions: () => [StreamLanguage.define(shellLanguage)],
  });

  const isEditMode = $derived(!!editingStep);

  const handleSubmit = () => {
    const submitted = form.toValue();
    if (submitted) onSubmit(submitted.nodeId, submitted.value);
  };
</script>

<div class="min-w-0 space-y-4">
  <div class="space-y-2">
    <Label for="node-id">{t(pipelineMessages.nodeId)}</Label>
    <Input
      id="node-id"
      bind:value={form.nodeId}
      placeholder={t(pipelineMessages.nodeIdPlaceholder)}
    />
  </div>

  <!-- A script and a command are alternatives, not both: this picker is what
       decides which half of the form is even asked for. -->
  <div class="grid grid-cols-2 gap-1 rounded-md border p-1">
    <button
      type="button"
      onclick={() => (form.mode = 'script')}
      class={cn(
        'rounded-sm px-3 py-1.5 text-sm font-medium transition-colors',
        form.mode === 'script'
          ? 'bg-primary text-primary-foreground'
          : 'text-muted-foreground hover:bg-accent',
      )}
    >
      {t(pipelineMessages.script)}
    </button>
    <button
      type="button"
      onclick={() => (form.mode = 'exec')}
      class={cn(
        'rounded-sm px-3 py-1.5 text-sm font-medium transition-colors',
        form.mode === 'exec'
          ? 'bg-primary text-primary-foreground'
          : 'text-muted-foreground hover:bg-accent',
      )}
    >
      {t(pipelineMessages.command)}
    </button>
  </div>

  {#if form.mode === 'script'}
    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="node-script">{t(pipelineMessages.script)}</Label>
        <div class="h-full w-full overflow-auto p-2">
          <div
            id="node-script"
            class="max-h-64 w-full"
            use:renderCodeMirror={{
              doc: form.script,
              extensions: editor.extensions,
              onView: editor.attach,
            }}
          ></div>
        </div>
      </div>
      <div class="space-y-2">
        <Label for="node-shell">{t(pipelineMessages.shell)}</Label>
        <Select
          type="single"
          value={form.shell}
          onValueChange={value => (form.shell = value as Shell)}
        >
          <SelectTrigger id="node-shell" class="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="sh" label="sh">sh</SelectItem>
            <SelectItem value="bash" label="bash">bash</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  {:else}
    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="node-command">{t(pipelineMessages.command)}</Label>
        <Input
          id="node-command"
          bind:value={form.command}
          placeholder={t(pipelineMessages.commandPlaceholder)}
        />
      </div>
      <div class="space-y-2">
        <Label>{t(pipelineMessages.argumentsLabel)}</Label>
        <div class="space-y-2">
          <!-- Keyed by index: an argument has no identity beyond its position. -->
          {#each form.args as arg, index (index)}
            <div class="flex items-center gap-2">
              <Input
                value={arg}
                oninput={event => form.setArg(index, event.currentTarget.value)}
                placeholder={t(pipelineMessages.argumentPlaceholder)}
              />
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                aria-label={t(pipelineMessages.removeArgument)}
                onclick={() => form.removeArg(index)}
              >
                <Trash2Icon class="size-4" />
              </Button>
            </div>
          {/each}
          <Button type="button" variant="outline" size="sm" onclick={form.addArg}>
            <PlusIcon class="size-4" />
            {t(pipelineMessages.addArgument)}
          </Button>
        </div>
      </div>
    </div>
  {/if}

  <div class="space-y-2">
    <Label for="node-working-dir">{t(pipelineMessages.workingDirectory)}</Label>
    <Input
      id="node-working-dir"
      bind:value={form.workingDir}
      placeholder={t(pipelineMessages.workingDirectoryPlaceholder)}
    />
  </div>

  <div class="space-y-2">
    <Label>{t(pipelineMessages.environmentVariables)}</Label>
    <div class="space-y-2">
      {#each form.envRows as row, index (index)}
        <div class="flex items-center gap-2">
          <Input
            value={row.key}
            oninput={event => form.setEnv(index, { key: event.currentTarget.value })}
            placeholder={t(pipelineMessages.envKeyPlaceholder)}
            class="flex-1"
          />
          <Select
            type="single"
            value={row.kind}
            onValueChange={value => form.setEnv(index, { kind: value as EnvRow['kind'] })}
          >
            <SelectTrigger aria-label={t(pipelineMessages.envKindLabel)} class="w-28 shrink-0">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="literal" label={t(pipelineMessages.envKindLiteral)}>
                {t(pipelineMessages.envKindLiteral)}
              </SelectItem>
              <SelectItem value="secret" label={t(pipelineMessages.envKindSecret)}>
                {t(pipelineMessages.envKindSecret)}
              </SelectItem>
            </SelectContent>
          </Select>

          {#if row.kind === 'secret'}
            <!-- A free-text box when there is nothing to pick from: the project
                 may hold no secret yet, or the caller may not be allowed to
                 list them, and neither should block writing a reference. -->
            {#if secrets.length > 0 || row.secretRef}
              <Select
                type="single"
                value={row.secretRef}
                onValueChange={value => form.setEnv(index, { secretRef: value })}
              >
                <SelectTrigger aria-label={t(pipelineMessages.referenceASecret)} class="flex-1">
                  <SelectValue placeholder={t(pipelineMessages.referenceASecret)} />
                </SelectTrigger>
                <SelectContent>
                  {#if row.secretRef && !secrets.some(secret => secret.name === row.secretRef)}
                    <SelectItem value={row.secretRef} label={row.secretRef}>
                      {row.secretRef}
                    </SelectItem>
                  {/if}
                  {#each secrets as secret (secret.id)}
                    <SelectItem value={secret.name} label={secret.name}>{secret.name}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            {:else}
              <Input
                value={row.secretRef}
                oninput={event => form.setEnv(index, { secretRef: event.currentTarget.value })}
                placeholder={t(pipelineMessages.secretNamePlaceholder)}
                class="flex-1"
              />
            {/if}
          {:else}
            <Input
              value={row.value}
              oninput={event => form.setEnv(index, { value: event.currentTarget.value })}
              placeholder={t(pipelineMessages.envValuePlaceholder)}
              class="flex-1"
            />
          {/if}

          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label={t(pipelineMessages.removeVariable)}
            onclick={() => form.removeEnv(index)}
          >
            <Trash2Icon class="size-4" />
          </Button>
        </div>
      {/each}
      <Button type="button" variant="outline" size="sm" onclick={form.addEnv}>
        <PlusIcon class="size-4" />
        {t(pipelineMessages.addVariable)}
      </Button>
    </div>
  </div>
</div>

<DialogFooter>
  <Button type="button" variant="outline" onclick={onCancel}>
    {t(pipelineMessages.cancel)}
  </Button>
  <Button type="button" onclick={handleSubmit}>
    {isEditMode ? t(pipelineMessages.save) : t(pipelineMessages.addNode)}
  </Button>
</DialogFooter>
