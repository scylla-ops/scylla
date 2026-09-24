<script lang="ts">
  import LogoScylla from '@/assets/logo_scylla.png';
  import LogoScyllaDark from '@/assets/logo_scylla_dark.png';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shadcn';
  import { ScyllaLoadingScreen } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { LoginState } from '../../login.state.svelte.ts';
  import LoginForm from '../LoginForm/LoginForm.svelte';
  import { loginMessages } from '../login.messages.ts';

  // Built during initialisation: its mutation needs an owner.
  const state = new LoginState();
</script>

{#if state.isSuccess}
  <ScyllaLoadingScreen />
{:else}
  <div class="flex flex-col items-center">
    <!-- A dark variant swapped by CSS: `.dark` is set before the first paint, so no flash. -->
    <img src={LogoScylla} alt="Scylla" class="h-2/6 w-2/6 dark:hidden" />
    <img src={LogoScyllaDark} alt="Scylla" class="hidden h-2/6 w-2/6 dark:block" />

    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{t(loginMessages.title)}</CardTitle>
        <CardDescription>{t(loginMessages.description)}</CardDescription>
      </CardHeader>
      <CardContent>
        <LoginForm handleSubmit={state.submit} isPending={state.isPending} />
      </CardContent>
    </Card>
  </div>
{/if}
