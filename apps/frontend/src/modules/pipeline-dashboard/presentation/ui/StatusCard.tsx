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
import { useEffect, useState } from "react";
import { LoaderCircle, CircleCheckBig, CircleOff, RefreshCw } from 'lucide-react';

interface Pipeline {
    id: string;
    name: string;
    steps:
        {
            name: string;
            commands: string[];
            status: string;
        }[]
}

const StatusIcon = ({status}: {status: string}) => {
    switch (status) {
        case "SUCCESS":
            return <CircleCheckBig className="text-green-400"/>
        case "IN PROGRESS":
            return <LoaderCircle className="animate-spin text-amber-500"/>
        case "FAILED":
            return <CircleOff className="text-red-500"/>
        default:
            return <RefreshCw className="animate-spin text-gray-400"/>
    }
}

export const StatusCard = ({ pipeline }: { pipeline: Pipeline }) => {
    const [progress, setProgress] = useState(0);
    const [status, setStatus] = useState('Unknow');

    const handleProgress = () => {
        let stepEnd = 0;
        pipeline.steps.map((step:{ name: string; commands: string[]; status: string;}) => {
            if (step.status === 'SUCCESS') {
                stepEnd += 1;
            } else {
                setStatus(step.status)
            }
        })
        if (stepEnd === pipeline.steps.length) {
            setStatus("SUCCESS")
        }
        setProgress((stepEnd * 100) / pipeline.steps.length)
    }

    useEffect(() => {
        handleProgress();
    }, [pipeline]);

    return (
        <Card className="w-full max-w-sm">
            <CardHeader>
                <CardTitle>{pipeline.name || "Pipeline"}</CardTitle>
                <CardDescription>
                    Current estimed status and progress
                </CardDescription>
                <CardAction>
                    <Button variant="link">Details</Button>
                </CardAction>
            </CardHeader>
            <CardContent>
                <Progress value={progress}/>
            </CardContent>
            <CardFooter className="flex items-center gap-2">
                Status: {status}
                <StatusIcon status={status}/>
            </CardFooter>
        </Card>
    )
}
