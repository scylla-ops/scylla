<!--
  A real footer snippet. `createRawSnippet` renders its string once and never
  re-runs, so it cannot show that the footer tracks `isValid` — which is the
  only thing worth asserting about the footer.
-->
<script lang="ts">
  import ScyllaForm from './ScyllaForm.svelte';
  import { FormItemType, type FormItem, type FormValues } from './scylla-form.struct.ts';

  type Ids = 'username';

  interface Props {
    onSubmit?: (values: FormValues<Ids>) => void;
    isPending?: boolean;
  }

  let { onSubmit = () => {}, isPending = false }: Props = $props();

  const items: readonly FormItem<Ids>[] = [
    { id: 'username', type: FormItemType.Input, inputType: 'text', label: 'Username' },
  ];
</script>

<ScyllaForm {items} {onSubmit} {isPending}>
  {#snippet footer({ isValid, isPending: pending })}
    <button type="submit" disabled={!isValid || pending}>
      {isValid ? 'Ready' : 'Incomplete'}
    </button>
  {/snippet}
</ScyllaForm>
