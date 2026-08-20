import { Checkbox } from '@shadcn/checkbox.tsx';
import { Button, Label } from '@shadcn';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shadcn/collapsible.tsx';
import { useState } from 'react';

export type CheckboxNode<T extends string | number = string> = {
  id: T;
  label: string;
  children?: CheckboxNode<T>[];
};

interface CheckboxTreeProps<T extends string | number = string> {
  nodes: CheckboxNode<T>[];
  className?: string;
  onCheckedChange?: (checkedIds: T[]) => void;
  allDisabled?: boolean;
}

interface TreeNodeProps<T extends string | number = string> {
  nodes: CheckboxNode<T>[];
  className?: string;
  parentChecked?: boolean;
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

// Correction : Ajout de la contrainte | number manquante sur le composant TreeNode
const TreeNode = <T extends string | number>({
  nodes,
  parentChecked,
  className,
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
                  disabled={parentChecked === false}
                  onCheckedChange={val => onToggle(node.id, Boolean(val))}
                  checked={isChecked}
                  id={node.id.toString()} // Correction : Conversion string requise pour le DOM HTML
                />
                <Label
                  htmlFor={node.id.toString()} // Correction : Conversion string requise pour le DOM HTML
                  className='cursor-pointer text-sm font-medium leading-none capitalize select-none'
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
  onCheckedChange,
  allDisabled,
  className,
}: CheckboxTreeProps<T>) => {
  const [checkedMap, setCheckedMap] = useState<Record<T, boolean>>({} as Record<T, boolean>);

  const handleToggle = (id: T, checked: boolean) => {
    const targetNode = findNode(nodes, id);
    if (!targetNode) return;

    const nextMap = { ...checkedMap };

    if (!checked) {
      const idsToUpdate = getAllDescendantIds(targetNode);

      console.log(idsToUpdate);
      idsToUpdate.forEach(nodeId => {
        nextMap[nodeId] = false;
      });
    } else {
      nextMap[targetNode.id] = checked;
    }

    setCheckedMap(nextMap);

    const checkedIds = Object.keys(nextMap)
      .filter(key => nextMap[key as unknown as T])
      .map(key => {
        return (isNaN(Number(key)) ? key : Number(key)) as unknown as T;
      });

    onCheckedChange?.(checkedIds);
  };

  return (
    <TreeNode
      parentChecked={allDisabled ? false : undefined}
      nodes={nodes}
      className={className}
      checkedMap={checkedMap}
      onToggle={handleToggle}
    />
  );
};
