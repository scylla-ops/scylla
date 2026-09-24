<script lang="ts">
  import PlusIcon from '@lucide/svelte/icons/plus';
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import {
    Button,
    Input,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { DraftInput } from '../../../utils/trigger-form.utils.ts';
  import { triggersMessages } from '../../triggers.messages.ts';

  interface Props {
    inputs: DraftInput[];
    onChange: (inputs: DraftInput[]) => void;
    /** Webhook triggers may extract values from the payload via a JSON pointer. */
    allowJsonPointer: boolean;
  }

  let { inputs, onChange, allowJsonPointer }: Props = $props();

  const update = (index: number, patch: Partial<DraftInput>) =>
    onChange(inputs.map((input, i) => (i === index ? { ...input, ...patch } : input)));

  const remove = (index: number) => onChange(inputs.filter((_, i) => i !== index));

  const add = () => onChange([...inputs, { key: '', valueKind: 'literal', value: '' }]);
</script>

<!-- Editor for a trigger's repeated key → (literal | json-pointer) inputs. -->
<div class="space-y-2">
  <div class="flex items-center justify-between">
    <span class="text-sm font-medium">{t(triggersMessages.inputs)}</span>
    <Button type="button" variant="ghost" size="sm" onclick={add} class="gap-1">
      <PlusIcon class="size-4" />
      {t(triggersMessages.addInput)}
    </Button>
  </div>

  {#if inputs.length === 0}
    <p class="text-xs text-muted-foreground">{t(triggersMessages.inputsHint)}</p>
  {:else}
    <!--
      Keyed by index, and deliberately: a draft row has no id until it is saved,
      and two blank rows are indistinguishable. Rows are only ever appended or
      removed from the end of the user's attention, so an index key costs
      nothing here — unlike in a server-backed list.
    -->
    {#each inputs as input, index (index)}
      <div class="flex items-center gap-2">
        <Input
          placeholder="KEY"
          value={input.key}
          oninput={event => update(index, { key: event.currentTarget.value })}
          class="w-1/3 font-mono"
        />
        {#if allowJsonPointer}
          <Select
            type="single"
            value={input.valueKind}
            onValueChange={value =>
              update(index, { valueKind: value as DraftInput['valueKind'] })}
          >
            <SelectTrigger class="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="literal" label={t(triggersMessages.literal)}>
                {t(triggersMessages.literal)}
              </SelectItem>
              <SelectItem value="jsonPointer" label={t(triggersMessages.jsonPointer)}>
                {t(triggersMessages.jsonPointer)}
              </SelectItem>
            </SelectContent>
          </Select>
        {/if}
        <Input
          placeholder={input.valueKind === 'jsonPointer' ? '/after' : 'value'}
          value={input.value}
          oninput={event => update(index, { value: event.currentTarget.value })}
          class="flex-1"
        />
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onclick={() => remove(index)}
          class="size-8 shrink-0"
        >
          <Trash2Icon class="size-4 text-destructive" />
          <!-- Icon-only: the React original carried no accessible name. -->
          <span class="sr-only">{t(triggersMessages.removeInput)}</span>
        </Button>
      </div>
    {/each}
  {/if}
</div>
