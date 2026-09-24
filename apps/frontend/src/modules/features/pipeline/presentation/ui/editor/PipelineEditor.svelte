<script lang="ts">
  import { StreamLanguage } from '@codemirror/language';
  import { json } from '@codemirror/legacy-modes/mode/javascript';
  import { Card, Tabs, TabsContent } from '@shadcn';
  import { renderCodeMirror } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { PipelineStep } from '../../../domain/structs/pipeline.struct.ts';
  import { createPipelineScript } from '../../pipeline-script.state.svelte.ts';
  import PipelineBlueprint from './blueprint/PipelineBlueprint.svelte';
  import PipelineEditorHeader from './PipelineEditorHeader/PipelineEditorHeader.svelte';
  import { createScriptEditor } from './script-editor.svelte.ts';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    /** `edit` tracks the unsaved changes; `create` does not. */
    mode: 'create' | 'edit';
    submitLabel: string;
    onSubmit: (values: { name: string; steps: PipelineStep[] }) => void;
    /** A default draft, or a fetched pipeline. */
    initialScript?: string;
    projectId?: string;
    isSubmitPending?: boolean;
  }

  let {
    mode,
    submitLabel,
    onSubmit,
    initialScript,
    projectId,
    isSubmitPending = false,
  }: Props = $props();

  const doc = createPipelineScript({
    projectId: () => projectId ?? '',
    initialScript: () => initialScript,
  });

  const editor = createScriptEditor({
    value: () => doc.script,
    onChange: next => doc.setScript(next),
    extensions: () => [StreamLanguage.define(json)],
  });

  const handleSubmit = () => {
    if (!doc.isValid) return;
    onSubmit({ name: doc.pipelineName, steps: doc.steps });
  };
</script>

<!--
  The script is the document; the blueprint reads and writes its steps. An invalid
  script disables the blueprint. Both panels stay mounted, so CodeMirror keeps its scroll.
-->
<Tabs value="blueprint" class="flex h-full flex-col gap-4">
  <div class="flex w-full items-center justify-between gap-4">
    <PipelineEditorHeader
      onSubmit={handleSubmit}
      {submitLabel}
      {mode}
      submitDisabled={!doc.isValid}
      blueprintDisabled={!doc.isValid}
      isDirty={doc.isDirty}
      isSaving={isSubmitPending}
    />
  </div>

  <TabsContent value="scripting" class="h-full overflow-hidden">
    <div class="flex h-full flex-col gap-2">
      <div class="min-h-0 flex-1 overflow-auto p-2">
        <div
          class="h-full"
          use:renderCodeMirror={{
            doc: doc.script,
            extensions: editor.extensions,
            hasError: !!doc.parseError,
            onView: editor.attach,
          }}
        ></div>
      </div>
      {#if doc.parseError}
        <p class="text-sm text-destructive">
          {t(pipelineMessages.invalidJson)}: {doc.parseError}
        </p>
      {/if}
    </div>
  </TabsContent>

  <TabsContent value="blueprint" class="h-full">
    <Card class="h-full bg-card p-0">
      <PipelineBlueprint
        steps={doc.steps}
        pipelineName={doc.pipelineName}
        {projectId}
        onStepsChange={steps => doc.setSteps(steps)}
        onNameChange={name => doc.setName(name)}
      />
    </Card>
  </TabsContent>
</Tabs>
