<script lang="ts">
  import type { Permission } from '@platform/authz';
  import { Checkbox, Label } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import type { CheckboxNode } from './checkbox-tree.ts';

  interface Props {
    nodes: CheckboxNode[];
    checked: Set<Permission>;
    /** False disables the whole subtree — the parent is unchecked, or a write is in flight. */
    disabled: boolean;
    /** True while the parent chain is checked; the root level is always true. */
    parentChecked?: boolean;
    toggle: (id: Permission, checked: boolean) => void;
  }

  let { nodes, checked, disabled, parentChecked = true, toggle }: Props = $props();

  /**
   * One open flag per node, keyed by id. Expanded by default, as in React.
   *
   * One record for the whole tree rather than one per level: ids are unique
   * across the tree, so a shared map is the same state with one owner.
   */
  let collapsed = $state<Record<Permission, boolean>>({} as Record<Permission, boolean>);
</script>

<!--
  The permission tree, recursing through a snippet rather than through the
  component importing itself.

  Self-import is the Svelte 5 idiom that replaced `<svelte:self>`, but it is a
  one-module cycle and `depcruise`'s `no-circular` is an `error` gate — rightly,
  since it cannot tell this apart from two files that are really one. A snippet
  may reference itself, which gives the same recursion with no edge at all.

  The collapse is `$state` on a plain boolean rather than the `collapsible`
  primitive: it is one class toggle with no floating layer, no focus management
  and no ARIA beyond `aria-expanded`, and porting a Radix primitive to carry it
  would be a file to maintain for nothing.
-->
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
            <!--
              `aria-label` as well as the `<Label for>`: bits-ui renders the box as
              a `<button role="checkbox">`, and the label association that named
              Radix's input names nothing here — the control announced itself as a
              bare "checkbox". The label stays because it is still the click target.
            -->
            <Checkbox
              id={String(node.id)}
              aria-label={node.label}
              checked={isChecked}
              disabled={disabled || !chainChecked}
              onCheckedChange={value => toggle(node.id, value === true)}
            />
            <!-- No `capitalize`: labels arrive already cased by the caller. -->
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
