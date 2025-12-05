import { Button } from "@/modules/core/presentation/ui/shadcn"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/modules/core/presentation/ui/shadcn/card";
import { Progress } from "@/modules/core/presentation/ui/shadcn";
import type { PipelineResponse } from '@/generated/pipeline';
import { CircleCheckBig } from 'lucide-react';

const StatusIcon = ({status}: {status: string}) => {
    switch (status) {
        case "SUCCESS":
            return <CircleCheckBig className="text-green-400"/>
        default:
            return <CircleCheckBig className="text-green-400"/>
    }
}

export const StatusCard = ({ pipeline }: { pipeline: PipelineResponse }) => {
    const pipelineId = pipeline.pipelineId || "Unknown";
    const content = pipeline.content || "No content";
    const createdAt = pipeline.createdAt ? new Date(pipeline.createdAt).toLocaleDateString() : "Unknown";
    
    return (
        <Card className="w-full max-w-sm">
            <CardHeader>
                <CardTitle className="truncate">{pipelineId}</CardTitle>
                <CardDescription>
                    Created: {createdAt}
                </CardDescription>
                <CardAction>
                    <Button variant="link">Details</Button>
                </CardAction>
            </CardHeader>
            <CardContent>
                <div className="text-sm text-gray-600 line-clamp-3">{content}</div>
                <Progress value={100}/>
            </CardContent>
            <CardFooter className="flex items-center gap-2">
                Status: Ready
                <StatusIcon status="SUCCESS"/>
            </CardFooter>
        </Card>
    )
}

