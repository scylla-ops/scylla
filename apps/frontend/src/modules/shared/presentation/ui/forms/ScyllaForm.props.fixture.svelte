<!-- Pins `ScyllaForm`'s id generic — see `DataTable.fixture.svelte` for why. -->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import ScyllaForm from './ScyllaForm.svelte';
  import type { Ids } from './form.fixture.ts';

  let props: ComponentProps<typeof ScyllaForm<Ids>> = $props();
</script>

<ScyllaForm {...props} />
