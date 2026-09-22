<script lang="ts">
  import { FormDialog, FormItemType, type FormItem, type FormValues } from '@shared/presentation/ui-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    currentName: string;
    onSave: (name: string) => void;
  }

  let { open, onOpenChange, currentName, onSave }: Props = $props();

  // `pattern` rejects whitespace outright: a pipeline name travels into a URL
  // and into the breadcrumb, so a name with a space is not merely untidy.
  const items = $derived<readonly FormItem<'name'>[]>([
    {
      id: 'name',
      label: t(pipelineMessages.name),
      placeholder: t(pipelineMessages.namePlaceholder),
      type: FormItemType.Input,
      inputType: 'text',
      defaultValue: currentName,
      pattern: '^\\S+$',
    },
  ]);

  const handleSubmit = ({ name }: FormValues<'name'>) => {
    if (!name.trim()) return;
    onSave(name.trim());
    onOpenChange(false);
  };
</script>

<!-- Renaming the pipeline, from the start node that carries its name. -->
<FormDialog
  {open}
  {onOpenChange}
  title={t(pipelineMessages.pipelineName)}
  description={t(pipelineMessages.pipelineNameDescription)}
  {items}
  submitLabel={t(pipelineMessages.save)}
  onSubmit={handleSubmit}
/>
