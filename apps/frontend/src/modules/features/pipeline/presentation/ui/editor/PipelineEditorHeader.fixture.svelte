<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import { Tabs } from '@shadcn';
  import PipelineEditorHeader from './PipelineEditorHeader.svelte';

  let props: ComponentProps<typeof PipelineEditorHeader> = $props();
</script>

<Tabs value="scripting">
  <PipelineEditorHeader {...props} />
</Tabs>
