import { Button } from '@/modules/shared/presentation/ui/shadcn';
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn/card.tsx';
import type { PipelineResponse } from '@/generated/pipeline.ts';
import { EditIcon, PlayIcon } from 'lucide-react';
import StatusIndicator from '@/modules/shared/presentation/ui/status-indicator.tsx';
import { useNavigate } from 'react-router-dom';
import { PipelineChart } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineChart.tsx';
import { cn } from '@core/presentation/utils';

export type StatusCardProps = {
  pipeline: PipelineResponse;
  onClick?: () => void;
  selected?: boolean;
};

export const StatusCard = ({ pipeline, onClick, selected }: StatusCardProps) => {
  const navigate = useNavigate();

  const goToSettings = () => navigate(`/pipeline-creation/${pipeline.pipelineId}`);

  const date = new Date(pipeline.createdAt);

  return (
    <Card
      onClick={onClick}
      className={cn(
        'transition-colors duration-100 cursor-pointer',
        !selected && 'hover:bg-gray-50',
        selected && 'bg-blue-50 border-blue-200 ring-1 ring-blue-200',
      )}
    >
      <CardHeader>
        <CardTitle className='truncate'>{pipeline.name}</CardTitle>
        <CardDescription>Created: {date.toDateString()}</CardDescription>
        <CardAction>
          <StatusIndicator state='success' label='Success' />
        </CardAction>
      </CardHeader>
      <CardContent>
        <PipelineChart />
      </CardContent>
      <CardFooter className='flex gap-2 w-full justify-between'>
        <Button className={'mr-2'} onClick={() => {}}>
          <PlayIcon /> Run
        </Button>
        <Button variant='outline' onClick={goToSettings}>
          <EditIcon /> Edit
        </Button>
      </CardFooter>
    </Card>
  );
};
