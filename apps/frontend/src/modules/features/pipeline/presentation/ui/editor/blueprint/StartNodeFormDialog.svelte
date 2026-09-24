<script lang="ts">
  import { FormDialog, FormItemType, type FormItem, type FormValues } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    currentName: string;
    onSave: (name: string) => void;
  }

  let { open, onOpenChange, currentName, onSave }: Props = $props();

  // No whitespace: the name goes into a URL and the breadcrumb.
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

<FormDialog
  {open}
  {onOpenChange}
  title={t(pipelineMessages.pipelineName)}
  description={t(pipelineMessages.pipelineNameDescription)}
  {items}
  submitLabel={t(pipelineMessages.save)}
  onSubmit={handleSubmit}
/>
