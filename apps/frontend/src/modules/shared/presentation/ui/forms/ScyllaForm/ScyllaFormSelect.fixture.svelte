<script lang="ts">
  import ScyllaForm from './ScyllaForm.svelte';
  import { FormItemType, type FormItem, type FormValues } from '../scylla-form.struct.ts';

  interface Props {
    onSubmit: (values: FormValues<'plan'>) => void;
  }

  let { onSubmit }: Props = $props();

  const items: readonly FormItem<'plan'>[] = [
    {
      id: 'plan',
      type: FormItemType.Select,
      label: 'Plan',
      placeholder: 'Pick one',
      options: [
        { label: 'Free', value: 'free' },
        { label: 'Pro', value: 'pro' },
      ],
    },
  ];
</script>

<ScyllaForm {items} {onSubmit} buttonLabel="Save" />
