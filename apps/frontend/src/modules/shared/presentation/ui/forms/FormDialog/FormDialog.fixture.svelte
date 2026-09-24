<!-- Pins the id generic of `FormDialog`. -->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import FormDialog from './FormDialog.svelte';

  let props: ComponentProps<typeof FormDialog<'name'>> = $props();
</script>

<FormDialog {...props} />
