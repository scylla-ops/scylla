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

export type ScyllaFormProps = {
  items: FormItem[];
  className?: string;
  onSubmit: (values: FormChange[]) => void;
  buttonLabel: string;
};

export const ScyllaForm = ({ items, className, buttonLabel, onSubmit }: ScyllaFormProps) => {
  const [values, setValues] = useState<FormChange[]>(
    items.map(item => ({ id: item.id, value: '' })),
  );

  const handleChange = (id: string, value: string) => {
    setValues(prev => prev.map(field => (field.id === id ? { ...field, value } : field)));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(values);
  };

  return (
    <form onSubmit={handleSubmit}>
      <FieldGroup className={className}>
        {items.map(item => (
          <Field key={item.id} className='gap-1'>
            <FieldLabel htmlFor={item.id}>{item.label}</FieldLabel>

            {item.type === FormItemType.Input && (
              <Input
                placeholder={item.placeholder}
                className={item.className}
                id={item.id}
                type={item.inputType}
                value={values.find(v => v.id === item.id)?.value}
                onChange={e => handleChange(item.id, e.target.value)}
              />
            )}

            {item.type === FormItemType.Select && (
              <Select
                value={values.find(v => v.id === item.id)?.value}
                onValueChange={val => handleChange(item.id, val)}
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

      <div className='flex justify-end mt-8'>
        <Button type='submit'>{buttonLabel}</Button>
      </div>
    </form>
  );
};
