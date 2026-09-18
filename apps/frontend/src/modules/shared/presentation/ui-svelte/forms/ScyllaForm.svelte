<script lang="ts" generics="TId extends string">
  import type { Snippet } from 'svelte';
  import {
    Button,
    Field,
    FieldGroup,
    FieldLabel,
    Input,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@shadcn-svelte';
  import { createFormState } from './form-state.svelte.ts';
  import { FormItemType, type FormItem, type FormValues } from './scylla-form.struct.ts';

  interface Props {
    items: readonly FormItem<TId>[];
    class?: string;
    onSubmit: (values: FormValues<TId>) => void;
    isPending?: boolean;
    /** Replaces the default submit row; receives validity and pending state. */
    footer?: Snippet<[{ isValid: boolean; isPending: boolean }]>;
    buttonLabel?: string;
  }

  let { items, class: className, onSubmit, isPending = false, footer, buttonLabel }: Props =
    $props();

  const form = createFormState(() => items);

  const handleSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    onSubmit(form.values);
  };
</script>

<form onsubmit={handleSubmit} class="space-y-4">
  <FieldGroup class={className}>
    {#each items as item, index (item.id)}
      <Field class="gap-1">
        <FieldLabel for={item.id}>{item.label}</FieldLabel>

        {#if item.type === FormItemType.Input}
          <Input
            id={item.id}
            type={item.inputType}
            disabled={isPending || item.disabled}
            placeholder={item.placeholder}
            class={item.class}
            autofocus={index === 0}
            value={form.values[item.id]}
            oninput={event => form.handleChange(item.id, event.currentTarget.value)}
          />
        {:else}
          <!--
            `value` + `onValueChange` rather than `bind:value`: the values live
            in the form state, which owns reset and validity, and binding would
            put a second writer on the same field.
          -->
          <Select
            type="single"
            value={form.values[item.id]}
            onValueChange={value => form.handleChange(item.id, value)}
            disabled={isPending || item.disabled}
          >
            <SelectTrigger id={item.id} class={item.class ?? 'w-full'}>
              <SelectValue placeholder={item.placeholder} />
            </SelectTrigger>
            <SelectContent>
              {#each item.options as option (option.value)}
                <SelectItem value={option.value} label={option.label}>{option.label}</SelectItem>
              {/each}
            </SelectContent>
          </Select>
        {/if}
      </Field>
    {/each}
  </FieldGroup>

  {#if footer}
    {@render footer({ isValid: form.isValid, isPending })}
  {:else}
    <div class="mt-8 flex justify-end">
      <Button type="submit" disabled={!form.isValid || isPending}>{buttonLabel}</Button>
    </div>
  {/if}
</form>
