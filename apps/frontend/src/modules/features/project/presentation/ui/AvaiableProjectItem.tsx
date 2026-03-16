import { Building2 } from 'lucide-react';

type AvailableProjectItemProps = {
  name: string;
};

//TODO: see dropdownmenushortcut componnent for user convenience
export const AvailableProjectItem = ({ name }: AvailableProjectItemProps) => {
  return (
    <div className={'flex items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-sidebar-accent'}>
      <div className='flex size-6 items-center justify-center rounded-md border'>
        <Building2 className='size-3.5 shrink-0' />
      </div>
      <span className='flex-1 truncate'>{name}</span>
    </div>
  );
};

export default AvailableProjectItem;
