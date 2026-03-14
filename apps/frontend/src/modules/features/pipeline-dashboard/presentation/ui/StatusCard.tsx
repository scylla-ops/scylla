import { Button } from '@shadcn';
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@shadcn/card.tsx';
import type { PipelineResponse } from '@/generated/pipeline.ts';
import { PlayIcon, SettingsIcon } from 'lucide-react';
import StatusIndicator from '@/modules/shared/presentation/status-indicator.tsx';
import { useNavigate } from 'react-router-dom';
import { PipelineChart } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineChart.tsx';

export type StatusCardProps = {
  pipeline: PipelineResponse;
};

export const StatusCard = ({ pipeline }: StatusCardProps) => {
  const navigate = useNavigate();
  const createdAt = '10/10/2023';

  const goToSettings = () => navigate(`/pipeline-creation/${pipeline.pipelineId}`);

  return (
    <Card className='hover:bg-gray-50 transition-colors duration-100'>
      <CardHeader>
        <CardTitle className='truncate'>{pipeline.name}</CardTitle>
        <CardDescription>Created: {createdAt}</CardDescription>
        <CardAction>
          <StatusIndicator state={'success'} label={'Success'} />
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
          <SettingsIcon /> Settings
        </Button>
      </CardFooter>
    </Card>
  );
};
