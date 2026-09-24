<script lang="ts">
  import { createMutation, createQuery } from '@platform/query';
  import {
    Avatar,
    AvatarFallback,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '@shadcn';
  import {
    FormItemType,
    ScyllaForm,
    type FormItem,
    type FormValues,
  } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userMutations, userQueries } from '../../user.queries.ts';
  import { userMessages } from '../user.messages.ts';

  interface Props {
    userId?: string;
  }

  let { userId }: Props = $props();

  const userQuery = createQuery(() => userQueries.byId(userId || undefined));
  const updateUser = createMutation(() => userMutations.update());

  const user = $derived(userQuery.data);

  const items: readonly FormItem<'username' | 'user-id'>[] = $derived([
    {
      label: t(userMessages.username),
      placeholder: user?.username ?? '',
      id: 'username',
      type: FormItemType.Input,
      inputType: 'text',
      disabled: false,
      defaultValue: user?.username ?? '',
    },
    {
      label: t(userMessages.userId),
      placeholder: user?.userId ?? '',
      id: 'user-id',
      type: FormItemType.Input,
      inputType: 'text',
      disabled: true,
      defaultValue: user?.userId ?? '',
    },
  ]);

  const handleSubmit = ({ username }: FormValues<'username' | 'user-id'>) => {
    if (userId) updateUser.mutate({ userId, username });
  };

  const initials = (username: string) =>
    username
      .split(' ')
      .map(part => part[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
</script>

<Card class="w-full bg-card">
  <CardHeader>
    <CardTitle>{t(userMessages.userInformation)}</CardTitle>
    <CardDescription>{t(userMessages.manageAccount)}</CardDescription>
  </CardHeader>

  <!--
    One card, four bodies — the React version repeated the whole card in each
    branch, so the header existed four times and could drift.
  -->
  {#if !userId}
    <CardContent>
      <div class="text-center text-muted-foreground">
        {t(userMessages.informationUnavailable)}
      </div>
    </CardContent>
  {:else if userQuery.isLoading}
    <CardContent>
      <div class="text-center text-muted-foreground">{t(userMessages.loadingInformation)}</div>
    </CardContent>
  {:else if userQuery.isError || !user}
    <CardContent>
      <div class="text-center text-destructive">{t(userMessages.informationError)}</div>
    </CardContent>
  {:else}
    <CardContent class="space-y-4">
      <div class="flex items-center space-x-4">
        <Avatar>
          <AvatarFallback>{initials(user.username)}</AvatarFallback>
        </Avatar>
        <div>
          <div class="text-base font-medium">{user.username}</div>
          <div class="text-sm text-muted-foreground">{t(userMessages.activeAccount)}</div>
        </div>
      </div>

      <!--
        Keyed on the user: `createFormState` seeds its values once, so without
        this the fields would keep showing the previous account's name when the
        route moves to another user.
      -->
      {#key user.userId}
        <ScyllaForm
          onSubmit={handleSubmit}
          {items}
          buttonLabel={t(userMessages.save)}
          class="gap-2"
          isPending={updateUser.isPending}
        />
      {/key}
    </CardContent>
  {/if}
</Card>
