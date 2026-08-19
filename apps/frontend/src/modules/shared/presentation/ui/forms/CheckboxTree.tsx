import { Checkbox } from '@shadcn/checkbox.tsx';
import { Button, Label } from '@shadcn';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shadcn/collapsible.tsx';
import { useEffect, useState } from 'react';

export type CheckboxNode = {
  id: string;
  label: string;
  children?: CheckboxNode[];
};

interface CheckboxTreeProps {
  nodes: CheckboxNode[];
  className?: string;
  parentChecked?: boolean;
}

export const CheckboxTree = ({ nodes, parentChecked, className }: CheckboxTreeProps) => {
  const [checked, setChecked] = useState<Record<string, boolean>>({});

  useEffect(() => {
    if (parentChecked === false) {
      setChecked({});
    }
  }, [parentChecked, setChecked]);

  return (
    <div className='flex flex-col gap-1'>
      {nodes.map(node => {
        const hasChildren = Boolean(node.children && node.children.length > 0);

        const isChecked = parentChecked === false ? false : Boolean(checked[node.id]);

        return (
          <Collapsible key={node.id} className={className}>
            <div className='flex items-center gap-1.5 rounded-md px-1.5 py-1 cursor-pointer transition-colors hover:bg-muted/60'>
              {hasChildren ? (
                <CollapsibleTrigger asChild>
                  <Button
                    variant={'outline'}
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
                  onCheckedChange={val =>
                    setChecked(prev => ({ ...prev, [node.id]: val as boolean }))
                  }
                  checked={isChecked}
                  id={node.id}
                />
                <Label
                  htmlFor={node.id}
                  className='cursor-pointer text-sm font-medium leading-none capitalize select-none'
                >
                  {node.label}
                </Label>
              </div>
            </div>

            {/* Children */}
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

                        <CheckboxTree parentChecked={isChecked} nodes={[child]} />
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
