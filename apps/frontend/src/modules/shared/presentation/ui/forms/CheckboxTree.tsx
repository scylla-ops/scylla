import { Checkbox } from '@shadcn/checkbox.tsx';
import { Button, Label } from '@shadcn';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shadcn/collapsible.tsx';
import { useEffect, useState } from 'react';

export type CheckboxNode<T extends string | number = string> = {
  id: T;
  label: string;
  children?: CheckboxNode<T>[];
};

interface CheckboxTreeProps<T extends string | number = string> {
  nodes: CheckboxNode<T>[];
  className?: string;
  /**
   * Ids checked on mount — the state to restore when editing an existing
   * record. Ids whose parent chain isn't checked are dropped, so the seeded
   * state always matches what the tree can actually render. Passing a set that
   * differs from the current selection re-seeds the tree, which makes it safe
   * to feed the value back from `onCheckedChange`.
   */
  defaultCheckedIds?: T[];
  onCheckedChange?: (checkedIds: T[]) => void;
  /** Disables every checkbox without altering the current selection. */
  allDisabled?: boolean;
}

interface TreeNodeProps<T extends string | number = string> {
  nodes: CheckboxNode<T>[];
  className?: string;
  parentChecked?: boolean;
  disabled?: boolean;
  checkedMap: Record<T, boolean>;
  onToggle: (id: T, checked: boolean) => void;
}

const getAllDescendantIds = <T extends string | number>(node: CheckboxNode<T>): T[] => {
  let ids: T[] = [node.id];
  if (node.children) {
    node.children.forEach(child => {
      ids = ids.concat(getAllDescendantIds(child));
    });
  }
  return ids;
};

const findNode = <T extends string | number>(
  nodes: CheckboxNode<T>[],
  id: T,
): CheckboxNode<T> | null => {
  for (const node of nodes) {
    if (node.id === id) return node;
    if (node.children) {
      const found = findNode(node.children, id);
      if (found) return found;
    }
  }
  return null;
};

const toCheckedMap = <T extends string | number>(ids: T[]): Record<T, boolean> => {
  const map = {} as Record<T, boolean>;
  ids.forEach(id => {
    map[id] = true;
  });
  return map;
};

/**
 * Walks the tree in render order and collects the ids that actually read as
 * checked: a node counts only when its own box and its whole parent chain are
 * checked. A node reachable through several parents (multiple `dependsOn`)
 * appears once.
 */
const collectCheckedIds = <T extends string | number>(
  nodes: CheckboxNode<T>[],
  checkedMap: Record<T, boolean>,
  collected: Set<T> = new Set<T>(),
): Set<T> => {
  nodes.forEach(node => {
    if (!checkedMap[node.id]) return;
    collected.add(node.id);
    if (node.children) collectCheckedIds(node.children, checkedMap, collected);
  });
  return collected;
};

const sameIds = <T extends string | number>(a: Set<T>, b: Set<T>): boolean =>
  a.size === b.size && [...a].every(id => b.has(id));

// Correction : Ajout de la contrainte | number manquante sur le composant TreeNode
const TreeNode = <T extends string | number>({
  nodes,
  parentChecked,
  className,
  disabled,
  checkedMap,
  onToggle,
}: TreeNodeProps<T>) => {
  return (
    <div className='flex flex-col gap-1'>
      {nodes.map(node => {
        const hasChildren = Boolean(node.children && node.children.length > 0);
        const isChecked = parentChecked === false ? false : Boolean(checkedMap[node.id]);

        return (
          <Collapsible defaultOpen={true} key={node.id} className={className}>
            <div className='flex items-center gap-1.5 rounded-md px-1.5 py-1 cursor-pointer transition-colors hover:bg-muted/60'>
              {hasChildren ? (
                <CollapsibleTrigger asChild>
                  <Button
                    variant='outline'
                    className='group z-10 flex h-5 w-5 shrink-0 items-center justify-center rounded-full p-0 text-xs font-mono text-muted-foreground ring-1 ring-border select-none hover:bg-muted hover:text-foreground'
                  >
                    <span className='group-data-[state=open]:hidden'>+</span>
                    <span className='hidden group-data-[state=open]:inline'>−</span>
                  </Button>
                </CollapsibleTrigger>
              ) : (
                <div className='h-5 w-5 shrink-0' />
              )}

              <div className='flex h-full w-full items-center gap-2'>
                <Checkbox
                  disabled={disabled || parentChecked === false}
                  onCheckedChange={val => onToggle(node.id, Boolean(val))}
                  checked={isChecked}
                  id={node.id.toString()}
                />
                <Label
                  htmlFor={node.id.toString()}
                  // No `capitalize`: labels arrive already cased by the caller.
                  className='cursor-pointer text-sm font-medium leading-none select-none'
                >
                  {node.label}
                </Label>
              </div>
            </div>

            {hasChildren && (
              <CollapsibleContent>
                <div className='relative flex flex-col pt-1'>
                  {node.children!.map((child, childIndex) => {
                    const isChildLast = childIndex === node.children!.length - 1;

                    return (
                      <div key={child.id} className='relative pl-11'>
                        {isChildLast ? (
                          <span
                            aria-hidden='true'
                            className='absolute left-14 top-0 h-3.5 w-4 rounded-bl-md border-l border-b border-border'
                          />
                        ) : (
                          <>
                            <span
                              aria-hidden='true'
                              className='absolute left-14 top-0 h-full w-px bg-border'
                            />
                            <span
                              aria-hidden='true'
                              className='absolute left-14 top-3.5 h-px w-4 bg-border'
                            />
                          </>
                        )}

                        <TreeNode
                          parentChecked={isChecked}
                          disabled={disabled}
                          nodes={[child]}
                          checkedMap={checkedMap}
                          onToggle={onToggle}
                        />
                      </div>
                    );
                  })}
                </div>
              </CollapsibleContent>
            )}
          </Collapsible>
        );
      })}
    </div>
  );
};

export const CheckboxTree = <T extends string | number>({
  nodes,
  defaultCheckedIds,
  onCheckedChange,
  allDisabled,
  className,
}: CheckboxTreeProps<T>) => {
  const [checkedMap, setCheckedMap] = useState<Record<T, boolean>>(() =>
    toCheckedMap([...collectCheckedIds(nodes, toCheckedMap(defaultCheckedIds ?? []))]),
  );

  // Re-seed when the caller hands over a different selection (another role, a
  // scope change). Feeding back the selection we just emitted is a no-op.
  useEffect(() => {
    setCheckedMap(prev => {
      const seeded = collectCheckedIds(nodes, toCheckedMap(defaultCheckedIds ?? []));
      return sameIds(collectCheckedIds(nodes, prev), seeded) ? prev : toCheckedMap([...seeded]);
    });
  }, [nodes, defaultCheckedIds]);

  const handleToggle = (id: T, checked: boolean) => {
    const targetNode = findNode(nodes, id);
    if (!targetNode) return;

    const nextMap = { ...checkedMap };

    if (!checked) {
      const idsToUpdate = getAllDescendantIds(targetNode);

      idsToUpdate.forEach(nodeId => {
        nextMap[nodeId] = false;
      });
    } else {
      nextMap[targetNode.id] = checked;
    }

    setCheckedMap(nextMap);

    onCheckedChange?.([...collectCheckedIds(nodes, nextMap)]);
  };

  return (
    <TreeNode
      nodes={nodes}
      className={className}
      disabled={allDisabled}
      checkedMap={checkedMap}
      onToggle={handleToggle}
    />
  );
};
