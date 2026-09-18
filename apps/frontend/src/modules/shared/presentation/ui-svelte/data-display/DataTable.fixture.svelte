<!--
  Pins `DataTable`'s row type so the test does not have to.

  `render(Component, props)` gives TypeScript nowhere to infer a generic from,
  and the obvious fix — an instantiation expression, `DataTable<Datum>` — is
  available only to `svelte-check`: `tsc -b` resolves every `.svelte` import to
  Svelte's ambient `LegacyComponentType`, which has no type parameters, and
  fails with TS2635. A fixture moves the instantiation *into* a `.svelte` file,
  where `tsc` never looks and `svelte-check` checks it properly, and the test
  renders something plainly typed.
-->
<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import DataTable from './DataTable.svelte';
  import type { Datum } from './data-table.fixture.ts';

  let props: ComponentProps<typeof DataTable<Datum>> = $props();
</script>

<DataTable {...props} />
