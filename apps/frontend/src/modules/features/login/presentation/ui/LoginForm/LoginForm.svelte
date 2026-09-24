<script lang="ts">
  import { Button } from '@shadcn';
  import { ScyllaForm, FormItemType, type FormItem } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { loginMessages } from '../login.messages.ts';

  interface Props {
    handleSubmit: (login: string, password: string) => void;
    isPending?: boolean;
  }

  let { handleSubmit, isPending = false }: Props = $props();

  const items: readonly FormItem<'username' | 'password'>[] = $derived([
    {
      id: 'username',
      label: t(loginMessages.username),
      placeholder: t(loginMessages.usernamePlaceholder),
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'password',
      label: t(loginMessages.password),
      placeholder: t(loginMessages.passwordPlaceholder),
      type: FormItemType.Input,
      inputType: 'password',
    },
  ]);
</script>

<ScyllaForm
  {items}
  class="gap-4"
  onSubmit={values => handleSubmit(values.username, values.password)}
  {isPending}
>
  {#snippet footer({ isValid, isPending: pending })}
    <Button type="submit" class="mt-2 w-full" disabled={!isValid || pending}>
      {t(loginMessages.submit)}
    </Button>
  {/snippet}
</ScyllaForm>
