<script lang="ts">
  import LogoScylla from '@/assets/logo_scylla.png';
  import {
    Button,
    Card,
    CardAction,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
  } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { marketplaceMessages } from './marketplace.messages.ts';

  interface Props {
    provider: string;
    title: string;
    /** Spelt as the proto field is — see the struct. */
    descrption: string;
    class?: string;
  }

  let { provider, title, descrption, class: className }: Props = $props();
</script>

<Card class={cn('flex h-fit min-h-[250px] flex-col gap-2 bg-card', className)}>
  <CardHeader class="flex items-center justify-between">
    <img src={LogoScylla} alt="logo" class="h-24 w-24" />
    <CardAction>
      <CardDescription>{provider}</CardDescription>
    </CardAction>
  </CardHeader>

  <CardContent class="flex-1">
    <CardTitle class="text-2xl">{title}</CardTitle>
    <p>{descrption}</p>
  </CardContent>

  <CardFooter class="flex justify-end gap-2">
    <Button variant="outline">{t(marketplaceMessages.learnMore)}</Button>
    <Button>{t(marketplaceMessages.download)}</Button>
  </CardFooter>
</Card>
