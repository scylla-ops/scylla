<script lang="ts">
  import type { Permission } from '@platform/authz';
  import { Checkbox, Label } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import type { CheckboxNode } from './checkbox-tree.ts';

  interface Props {
    nodes: CheckboxNode[];
    checked: Set<Permission>;
    /** False disables the subtree: unchecked parent, or a write in flight. */
    disabled: boolean;
    /** The root level is always true. */
    parentChecked?: boolean;
    toggle: (id: Permission, checked: boolean) => void;
  }

  let { nodes, checked, disabled, parentChecked = true, toggle }: Props = $props();

  let collapsed = $state<Record<Permission, boolean>>({} as Record<Permission, boolean>);
</script>

<!-- Recursion through a snippet: a component importing itself is a cycle for `depcruise`. -->
{#snippet level(levelNodes: CheckboxNode[], chainChecked: boolean)}
  <div class="flex flex-col gap-1">
    {#each levelNodes as node (node.id)}
      {@const hasChildren = (node.children?.length ?? 0) > 0}
      {@const isChecked = chainChecked && checked.has(node.id)}
      {@const isOpen = !collapsed[node.id]}
      <div>
        <div
          class="flex items-center gap-1.5 rounded-md px-1.5 py-1 transition-colors hover:bg-muted/60"
        >
          {#if hasChildren}
            <button
              type="button"
              aria-expanded={isOpen}
              aria-label={isOpen
                ? t(rolesMessages.hideSubPermissions(node.label))
                : t(rolesMessages.showSubPermissions(node.label))}
              onclick={() => (collapsed[node.id] = isOpen)}
              class="z-10 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border p-0 font-mono text-xs text-muted-foreground ring-1 ring-border select-none hover:bg-muted hover:text-foreground"
            >
              {isOpen ? '−' : '+'}
            </button>
          {:else}
            <div class="h-5 w-5 shrink-0"></div>
          {/if}

          <div class="flex h-full w-full items-center gap-2">
            <!-- `aria-label` too: bits-ui renders a `<button role="checkbox">` that the `<Label for>` does not name. -->
            <Checkbox
              id={String(node.id)}
              aria-label={node.label}
              checked={isChecked}
              disabled={disabled || !chainChecked}
              onCheckedChange={value => toggle(node.id, value === true)}
            />
            <Label
              for={String(node.id)}
              class="cursor-pointer text-sm leading-none font-medium select-none"
            >
              {node.label}
            </Label>
          </div>
        </div>

        {#if hasChildren && isOpen}
          <div class="relative flex flex-col pt-1">
            {#each node.children ?? [] as child, index (child.id)}
              {@const isLast = index === (node.children?.length ?? 0) - 1}
              <div class="relative pl-11">
                {#if isLast}
                  <span
                    aria-hidden="true"
                    class="absolute left-14 top-0 h-3.5 w-4 rounded-bl-md border-b border-l border-border"
                  ></span>
                {:else}
                  <span
                    aria-hidden="true"
                    class="absolute left-14 top-0 h-full w-px bg-border"
                  ></span>
                  <span aria-hidden="true" class="absolute left-14 top-3.5 h-px w-4 bg-border"></span>
                {/if}

                {@render level([child], isChecked)}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/snippet}

{@render level(nodes, parentChecked)}
