import { Button } from '@/modules/core/presentation/ui/shadcn';
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/modules/core/presentation/ui/shadcn/card';
import type { PipelineResponse } from '@/generated/pipeline';
import { PlayIcon, SettingsIcon } from 'lucide-react';
import StatusIndicator from '@shadcn/status-indicator.tsx';

type StatusPipeline = 'success' | 'failure' | 'running';

export const StatusCard = ({ pipeline }: { pipeline: PipelineResponse }) => {
  const pipelineId = pipeline.pipelineId || 'Unknown';
  const content: StatusPipeline = pipeline.content as StatusPipeline;
  const createdAt = pipeline.createdAt
    ? new Date(pipeline.createdAt).toLocaleDateString()
    : 'Unknown';

  return (
    <Card className='w-full max-w-sm hover:bg-gray-50 transition-colors duration-100'>
      <CardHeader>
        <CardTitle className='truncate'>{pipelineId}</CardTitle>
        <CardDescription>Created: {createdAt}</CardDescription>
        <CardAction>
          <StatusIndicator state={content} label={content} />
        </CardAction>
      </CardHeader>
      <CardContent>no data yet</CardContent>
      <CardFooter className='flex gap-2 w-full justify-between'>
        <Button className={'mr-2'} onClick={() => {}}>
          <PlayIcon /> Run
        </Button>
        <Button variant='outline' onClick={() => {}}>
          <SettingsIcon /> Settings
        </Button>
      </CardFooter>
    </Card>
  );
};
