import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@shadcn';
import { type FormItem, FormItemType } from '@core/presentation/models/ScyllaForm.ts';
import { ScyllaForm } from '@/modules/shared/presentation/ScyllaForm.tsx';

const FormItems: FormItem[] = [
  {
    label: 'Alpha organization',
    id: 'a-organization',
    type: FormItemType.Select,
    placeholder: 'Select a value',
    options: [
      {
        label: 'Owner',
        value: 'owner',
      },
    ],
  },
  {
    label: 'Beta organization',
    id: 'b-organization',
    type: FormItemType.Select,
    placeholder: 'Select a value',
    options: [
      {
        label: 'Owner',
        value: 'owner',
      },
    ],
  },
  {
    label: 'Zeta organization',
    id: 'z-organization',
    type: FormItemType.Select,
    placeholder: 'Select a value',
    options: [
      {
        label: 'Owner',
        value: 'owner',
      },
    ],
  },
];
//TODO: refactor this and use the ScyllaForm component (maybe add the option to it to display field in flex-row with label)
export const Organizations = () => {
  return (
    <Card className='w-full bg-white'>
      <CardHeader>
        <CardTitle>Organizations</CardTitle>
        <CardDescription>Manage your organizations.</CardDescription>
      </CardHeader>

      <CardContent className='space-y-4'>
        <ScyllaForm items={FormItems} onSubmit={() => {}} buttonLabel={'Save'} />
      </CardContent>
    </Card>
  );
};
