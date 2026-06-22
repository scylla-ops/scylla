import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { Plus, Trash2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import type { DraftInput } from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';

interface TriggerInputsEditorProps {
  inputs: DraftInput[];
  onChange: (inputs: DraftInput[]) => void;
  /** Webhook triggers may extract values from the payload via a JSON pointer. */
  allowJsonPointer: boolean;
}

/** Editor for a trigger's repeated key → (literal | json-pointer) inputs. */
export const TriggerInputsEditor = ({
  inputs,
  onChange,
  allowJsonPointer,
}: TriggerInputsEditorProps) => {
  const update = (index: number, patch: Partial<DraftInput>) =>
    onChange(inputs.map((input, i) => (i === index ? { ...input, ...patch } : input)));
  const remove = (index: number) => onChange(inputs.filter((_, i) => i !== index));
  const add = () => onChange([...inputs, { key: '', valueKind: 'literal', value: '' }]);

  return (
    <div className='space-y-2'>
      <div className='flex items-center justify-between'>
        <span className='text-sm font-medium'>
          <Trans>Inputs</Trans>
        </span>
        <Button type='button' variant='ghost' size='sm' onClick={add} className='gap-1'>
          <Plus className='size-4' />
          <Trans>Add input</Trans>
        </Button>
      </div>

      {inputs.length === 0 ? (
        <p className='text-xs text-muted-foreground'>
          <Trans>Optional values injected into the run as environment variables.</Trans>
        </p>
      ) : (
        inputs.map((input, index) => (
          <div key={index} className='flex items-center gap-2'>
            <Input
              placeholder='KEY'
              value={input.key}
              onChange={e => update(index, { key: e.target.value })}
              className='w-1/3 font-mono'
            />
            {allowJsonPointer && (
              <Select
                value={input.valueKind}
                onValueChange={value => update(index, { valueKind: value as DraftInput['valueKind'] })}
              >
                <SelectTrigger className='w-36'>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='literal'>
                    <Trans>Literal</Trans>
                  </SelectItem>
                  <SelectItem value='jsonPointer'>JSON pointer</SelectItem>
                </SelectContent>
              </Select>
            )}
            <Input
              placeholder={input.valueKind === 'jsonPointer' ? '/after' : 'value'}
              value={input.value}
              onChange={e => update(index, { value: e.target.value })}
              className='flex-1'
            />
            <Button
              type='button'
              variant='ghost'
              size='icon'
              onClick={() => remove(index)}
              className='size-8 shrink-0'
            >
              <Trash2 className='size-4 text-destructive' />
            </Button>
          </div>
        ))
      )}
    </div>
  );
};
