<script lang="ts">
  import { Avatar, AvatarFallback, AvatarImage } from '../index.ts';

  interface Props {
    src?: string;
  }

  let { src }: Props = $props();
</script>

<Avatar>
  <AvatarImage {src} alt="Ada Lovelace" />
  <AvatarFallback>AL</AvatarFallback>
</Avatar>
