<script lang="ts">
  import scyllaLogo from '@/assets/logo_scylla.png';
  import scyllaLogoDark from '@/assets/logo_scylla_dark.png';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shadcn-svelte';
  import { ScyllaForm, type FormValues } from '@shared/presentation/ui-svelte';
  import { activeLocale, t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { createOrganizationItems } from '@/modules/features/organization';
  import { layoutMessages } from './layout.messages.ts';
  import { welcomeIn } from './welcome-transition.ts';

  interface Props {
    isPending: boolean;
    onSubmit: (values: FormValues<'name' | 'description'>) => void;
  }

  let { isPending, onSubmit }: Props = $props();

  const items = $derived((activeLocale(), createOrganizationItems()));
</script>

<main in:welcomeIn class="flex h-full w-full flex-col p-2">
  <div class="flex h-full min-h-screen w-full flex-col items-center bg-background">
    <img src={scyllaLogo} alt="Scylla" class="h-2/6 w-2/6 dark:hidden" />
    <img src={scyllaLogoDark} alt="Scylla" class="hidden h-2/6 w-2/6 dark:block" />
    <Card class="w-full max-w-md">
      <CardHeader class="text-center">
        <CardTitle class="text-2xl">{t(layoutMessages.welcome)}</CardTitle>
        <CardDescription>{t(layoutMessages.getStarted)}</CardDescription>
      </CardHeader>
      <CardContent>
        <ScyllaForm {items} {isPending} {onSubmit} buttonLabel={t(layoutMessages.create)} />
      </CardContent>
    </Card>
  </div>
</main>
