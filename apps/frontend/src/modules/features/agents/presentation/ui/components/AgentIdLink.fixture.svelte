<script lang="ts">
  import AgentIdLink from './AgentIdLink.svelte';

  interface Props {
    id: string;
    onCardClick: () => void;
  }

  let { id, onCardClick }: Props = $props();
</script>

<!--
  The card wrapper the component actually lives in, so the test can pin that a
  copy click does not also open the agent. A `createRawSnippet` cannot do this:
  the child is a component, not markup.
-->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div onclick={onCardClick}>
  <AgentIdLink {id} />
</div>
