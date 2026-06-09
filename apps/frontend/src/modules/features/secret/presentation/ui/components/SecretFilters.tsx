import { LayoutGrid, List } from 'lucide-react';
import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import type { CredentialKind } from '@/modules/features/secret/domain/entities/credential.model.ts';
import type { CredentialsView } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';

interface CredentialsFiltersProps {
  search: string;
  onSearchChange: (value: string) => void;
  kindFilter: 'ALL' | CredentialKind;
  onKindFilterChange: (value: 'ALL' | CredentialKind) => void;
  view: CredentialsView;
  onViewChange: (view: CredentialsView) => void;
}

export const SecretFilters = ({
  search,
  onSearchChange,
  kindFilter,
  onKindFilterChange,
  view,
  onViewChange,
}: CredentialsFiltersProps) => {
  return (
    <div className='rounded-xl border border-border/60 bg-card p-4 flex flex-col gap-4 md:flex-row md:items-center md:gap-3'>
      <Input
        value={search}
        onChange={event => onSearchChange(event.target.value)}
        placeholder='Quick find by ID, name, or kind...'
        className='h-10 flex-1 rounded-lg'
      />

      <Select
        value={kindFilter}
        onValueChange={value => onKindFilterChange(value as 'ALL' | CredentialKind)}
      >
        <SelectTrigger className='h-10 w-full rounded-lg md:w-auto md:min-w-35'>
          <SelectValue placeholder='All kinds' />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value='ALL'>All kinds</SelectItem>
          <SelectItem value='SSH_KEY'>SSH key</SelectItem>
          <SelectItem value='TOKEN'>Token</SelectItem>
          <SelectItem value='SECRET_TEXT'>Secret text</SelectItem>
          <SelectItem value='LOGIN'>Login</SelectItem>
        </SelectContent>
      </Select>

      <div className='flex items-center gap-2'>
        <Button
          type='button'
          variant={view === 'list' ? 'default' : 'outline'}
          size='icon'
          onClick={() => onViewChange('list')}
          className='h-10 w-10 rounded-lg'
        >
          <List className='size-4' />
        </Button>
        <Button
          type='button'
          variant={view === 'grid' ? 'default' : 'outline'}
          size='icon'
          onClick={() => onViewChange('grid')}
          className='h-10 w-10 rounded-lg'
        >
          <LayoutGrid className='size-4' />
        </Button>
      </div>
    </div>
  );
};
