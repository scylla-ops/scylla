import { Checkbox } from '@shadcn/checkbox.tsx';
import { Button, Label } from '@shadcn';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shadcn/collapsible.tsx';

export type CheckboxNode = {
  id: string;
  label: string;
  children?: CheckboxNode[];
};

interface CheckboxTreeProps {
  nodes: CheckboxNode[];
  className?: string;
}

export const CheckboxTree = ({ nodes, className }: CheckboxTreeProps) => {
  return (
    <div className='flex flex-col gap-1'>
      {nodes.map(node => {
        const hasChildren = Boolean(node.children && node.children.length > 0);

        return (
          <Collapsible key={node.id} className={className}>
            <div className='flex items-center gap-1.5 rounded-md px-1.5 py-1 cursor-pointer transition-colors hover:bg-muted/60'>
              {hasChildren ? (
                <CollapsibleTrigger asChild>
                  <Button
                    variant={'secondary'}
                    className='group z-10 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-background p-0 text-xs font-mono text-muted-foreground ring-1 ring-border select-none hover:bg-muted hover:text-foreground'
                  >
                    <span className='group-data-[state=open]:hidden'>+</span>
                    <span className='hidden group-data-[state=open]:inline'>−</span>
                  </Button>
                </CollapsibleTrigger>
              ) : (
                <div className='h-5 w-5 shrink-0' />
              )}

              <div className='flex h-full w-full items-center gap-2'>
                <Checkbox id={node.id} />
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
                {/* pas de gap ici : les traits verticaux de chaque enfant doivent se toucher pour former une ligne continue */}
                <div className='relative flex flex-col pt-1'>
                  {node.children!.map((child, childIndex) => {
                    const isChildLast = childIndex === node.children!.length - 1;

                    return (
                      /* pl-11 décale tout le bloc enfant vers la droite */
                      <div key={child.id} className='relative pl-11'>
                        {/* Le tronc est centré sur la colonne du bouton +/- (ou son espace réservé) :
                            le bouton (z-10) passe visuellement au-dessus de la ligne, qui n'a donc
                            plus besoin de s'étirer pour rejoindre la checkbox quand il n'y a pas de bouton */}
                        {isChildLast ? (
                          /* Dernier enfant : coin arrondi (└) qui referme proprement la ligne */
                          <span
                            aria-hidden='true'
                            className='absolute left-15 top-0 h-3.5 w-4 rounded-bl-md border-l border-b border-border'
                          />
                        ) : (
                          <>
                            {/* Ligne verticale continue (├) : rejoint directement le frère suivant, sans coupure */}
                            <span
                              aria-hidden='true'
                              className='absolute left-15 top-0 h-full w-px bg-border'
                            />
                            {/* Embranchement horizontal vers la checkbox de l'enfant */}
                            <span
                              aria-hidden='true'
                              className='absolute left-15 top-3.5 h-px w-4 bg-border'
                            />
                          </>
                        )}

                        <CheckboxTree nodes={[child]} />
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
