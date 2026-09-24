<script lang="ts">
  import AuthGuard from './Auth.guard.svelte';
</script>

<AuthGuard>
  <div data-testid="outlet"></div>
</AuthGuard>
