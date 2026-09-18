<!-- Pins `FormDialog`'s id generic — see `DataTable.fixture.svelte` for why. -->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import FormDialog from './FormDialog.svelte';

  let props: ComponentProps<typeof FormDialog<'name'>> = $props();
</script>

<FormDialog {...props} />
