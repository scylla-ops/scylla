<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from '../index.ts';
</script>

<Dialog>
  <DialogTrigger>Open</DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Create a secret</DialogTitle>
      <DialogDescription>It will be available to every pipeline.</DialogDescription>
    </DialogHeader>
  </DialogContent>
</Dialog>
