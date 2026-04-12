import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@shadcn';

import { type FormItem, FormItemType } from '@core/presentation/models/ScyllaForm.ts';
import { ScyllaForm } from '@shared/presentation/ui/ScyllaForm.tsx';
import { Trans } from '@lingui/react/macro';

const FormItems: FormItem[] = [
  {
    label: 'First name',
    placeholder: 'John',
    id: 'first-name',
    type: FormItemType.Input,
    inputType: 'text',
  },
  {
    label: 'Last name',
    placeholder: 'Doe',
    id: 'last-name',
    type: FormItemType.Input,
    inputType: 'text',
  },
  {
    label: 'Email',
    placeholder: 'example@gmail.com',
    id: 'email',
    type: FormItemType.Input,
    inputType: 'email',
  },
  {
    label: 'Password',
    placeholder: '******',
    id: 'password',
    type: FormItemType.Input,
    inputType: 'password',
  },
  {
    label: 'Telephone',
    placeholder: '+33 1 22 33 44 55',
    id: 'tel',
    type: FormItemType.Input,
    inputType: 'tel',
  },
  {
    label: 'Language',
    id: 'language',
    type: FormItemType.Select,
    placeholder: 'Select a language',
    options: [
      {
        label: 'English',
        value: 'english',
      },
      {
        label: 'French',
        value: 'french',
      },
    ],
  },
];

export const UserInformation = () => {
  return (
    <Card className='w-full bg-white'>
      <CardHeader>
        <CardTitle><Trans>User information</Trans></CardTitle>
        <CardDescription><Trans>Manage your account details.</Trans></CardDescription>
      </CardHeader>

      <CardContent className='space-y-4'>
        <div className='flex items-center space-x-4'>
          <Avatar>
            <AvatarImage src='https://github.com/YohannMgt.png' />
            <AvatarFallback>YM</AvatarFallback>
          </Avatar>
          <div>
            <div className='text-base font-medium'>Yohann Mangenot</div>
            <div className='text-sm text-gray-500'>Cloud plan</div>
          </div>
        </div>

        <ScyllaForm
          onSubmit={v => console.log(v)}
          items={FormItems}
          buttonLabel={'Save'}
          className={'gap-2'}
        />
      </CardContent>
    </Card>
  );
};
