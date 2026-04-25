import type { ReactNode } from 'react';
import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/modules/shared/presentation/ui/shadcn';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@core/presentation/models/ScyllaForm.ts';
import { Field, FieldGroup, FieldLabel } from '@/modules/shared/presentation/ui/shadcn/field.tsx';
import { useState } from 'react';

// --- Form state hook ---

//todo: move this into a separate file?
// eslint-disable-next-line react-refresh/only-export-components
export const useFormState = (items: FormItem[]) => {
  const [values, setValues] = useState<FormChange[]>(
    items.map(item => ({ id: item.id, value: '' })),
  );

  const handleChange = (id: string, value: string) => {
    setValues(prev => prev.map(field => (field.id === id ? { ...field, value } : field)));
  };

  const reset = () => setValues(items.map(item => ({ id: item.id, value: '' })));

  const isValid = values.every(v => v.value.trim().length > 0);

  return { values, handleChange, reset, isValid };
};

// --- ScyllaForm ---

export type ScyllaFormProps = {
  items: FormItem[];
  className?: string;
  onSubmit: (values: FormChange[]) => void;
  isPending?: boolean;
  footer?: (props: { isValid: boolean; isPending: boolean }) => ReactNode;
  buttonLabel?: ReactNode;
};

export const ScyllaForm = ({
  items,
  className,
  buttonLabel,
  onSubmit,
  isPending = false,
  footer,
}: ScyllaFormProps) => {
  const { values, handleChange, isValid } = useFormState(items);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(values);
  };

  return (
    <form onSubmit={handleSubmit} className='space-y-4'>
      <FieldGroup className={className}>
        {items.map((item, index) => (
          <Field key={item.id} className='gap-1'>
            <FieldLabel htmlFor={item.id}>{item.label}</FieldLabel>

            {item.type === FormItemType.Input && (
              <Input
                disabled={isPending || item.disabled}
                placeholder={item.placeholder}
                className={item.className}
                id={item.id}
                type={item.inputType}
                autoFocus={index === 0}
                value={values.find(v => v.id === item.id)?.value}
                onChange={e => handleChange(item.id, e.target.value)}
              />
            )}

            {item.type === FormItemType.Select && (
              <Select
                value={values.find(v => v.id === item.id)?.value}
                onValueChange={val => handleChange(item.id, val)}
                disabled={isPending || item.disabled}
              >
                <SelectTrigger className={item.className || 'w-full'} id={item.id}>
                  <SelectValue placeholder={item.placeholder} />
                </SelectTrigger>
                <SelectContent>
                  {item.options.map(option => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </Field>
        ))}
      </FieldGroup>

      {footer ? (
        footer({ isValid, isPending })
      ) : (
        <div className='flex justify-end mt-8'>
          <Button type='submit' disabled={!isValid || isPending}>
            {buttonLabel}
          </Button>
        </div>
      )}
    </form>
  );
};
