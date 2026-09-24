<script lang="ts">
  import ScyllaDialog from './ScyllaDialog.svelte';

  let { dismissible = true }: { dismissible?: boolean } = $props();

  let open = $state(true);
</script>

<button onclick={() => (open = true)}>Open</button>
<ScyllaDialog {open} onOpenChange={next => (open = next)} {dismissible} title="Invite">
  <input aria-label="Email" />
  <button onclick={() => (open = false)}>Done</button>
</ScyllaDialog>
