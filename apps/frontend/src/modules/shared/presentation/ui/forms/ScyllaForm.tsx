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
  type FormItem,
  FormItemType,
  type FormValues,
} from '@shared/presentation/structs/scylla-form.struct.ts';
import { Field, FieldGroup, FieldLabel } from '@/modules/shared/presentation/ui/shadcn/field.tsx';
import { useState } from 'react';

// --- Form state hook ---

const initialValues = <TId extends string>(items: readonly FormItem<TId>[]): FormValues<TId> =>
  Object.fromEntries(items.map(item => [item.id, item.defaultValue ?? ''])) as FormValues<TId>;

//todo: move this into a separate file?
// eslint-disable-next-line react-refresh/only-export-components
export const useFormState = <TId extends string>(items: readonly FormItem<TId>[]) => {
  const [values, setValues] = useState<FormValues<TId>>(() => initialValues(items));

  const handleChange = (id: TId, value: string) => {
    setValues(prev => ({ ...prev, [id]: value }));
  };

  const reset = () => setValues(initialValues(items));

  const isValid = items.every(item => {
    if (item.optional) return true;
    const value = values[item.id] ?? '';
    if (value.trim().length === 0) return false;
    if (item.type !== FormItemType.Input || !item.pattern) return true;
    return new RegExp(item.pattern).test(value);
  });

  return { values, handleChange, reset, isValid };
};

// --- ScyllaForm ---

export type ScyllaFormProps<TId extends string> = {
  items: readonly FormItem<TId>[];
  className?: string;
  onSubmit: (values: FormValues<TId>) => void;
  isPending?: boolean;
  footer?: (props: { isValid: boolean; isPending: boolean }) => ReactNode;
  buttonLabel?: ReactNode;
};

export const ScyllaForm = <TId extends string>({
  items,
  className,
  buttonLabel,
  onSubmit,
  isPending = false,
  footer,
}: ScyllaFormProps<TId>) => {
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
                value={values[item.id]}
                onChange={e => handleChange(item.id, e.target.value)}
              />
            )}

            {item.type === FormItemType.Select && (
              <Select
                value={values[item.id]}
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
