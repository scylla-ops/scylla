<!--
  Pins the generic row type: `tsc` cannot type `render(DataTable, …)` with a
  generic component.
-->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import DataTable from './DataTable.svelte';
  import type { Datum } from './data-table.fixture.ts';

  let props: ComponentProps<typeof DataTable<Datum>> = $props();
</script>

<DataTable {...props} />
