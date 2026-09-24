<script lang="ts">
  import LogoScylla from '@/assets/logo_scylla.png';
  import LogoScyllaDark from '@/assets/logo_scylla_dark.png';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shadcn';
  import { ScyllaLoadingScreen } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { LoginState } from '../login.state.svelte.ts';
  import LoginForm from './LoginForm.svelte';
  import { loginMessages } from './login.messages.ts';

  // Built here, in the component's initialisation: the mutation inside it
  // installs an effect, which needs an owner.
  const state = new LoginState();
</script>

{#if state.isSuccess}
  <ScyllaLoadingScreen />
{:else}
  <div class="flex flex-col items-center">
    <!--
      The wordmark is flat black and unreadable on the dark background, so the
      dark variant is the white cut of the same logo.

      Swapped by CSS rather than by reading the theme store: `index.html` puts
      `.dark` on <html> before the first paint, so this is right from the start,
      where a JS swap would paint the wrong logo first and flash.
    -->
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
