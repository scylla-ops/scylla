import { PipelineChart } from "./PipelineChart"
import { StatusCard } from "./StatusCard"

const PIPELINES = [
    {
        id: "58ba122d-9aac-4b38-93ec-ad7ddb10d238",
        name: "pipeline 1",
        steps: [
            {
                name: "Build",
                commands: ["echo 'Un\ndeux\ntrois'"],
                status: "SUCCESS",
            },
            {
                name: "Deploy",
                commands: [
                "echo Deploy"
                ],
                status: "IN PROGRESS"
            }
        ]
    },
    {
        id: "58ba122d-9aac-4b38-93ec-ad7ddb10d239",
        name: "pipeline 2",
        steps: [
            {
                name: "Build",
                commands: ["echo 'Un\ndeux\ntrois'"],
                status: "SUCCESS",
            },
            {
                name: "Deploy",
                commands: [
                "echo Deploy"
                ],
                status: "SUCCESS"
            },
            {
                name: "Test",
                commands: [
                "echo Test"
                ],
                status: "FAILED"
            }
        ]
    },
{
        id: "58ba122d-9aac-4b38-93ec-ad7ddb10d240",
        name: "pipeline 3",
        steps: [
            {
                name: "Build",
                commands: ["echo 'Un\ndeux\ntrois'"],
                status: "SUCCESS",
            },
            {
                name: "Deploy",
                commands: [
                "echo Deploy"
                ],
                status: "SUCCESS"
            },
            {
                name: "Test",
                commands: [
                "echo Test"
                ],
                status: "SUCCESS"
            }
        ]
    },
]


export const DashboardPipelinePage = () => {
    return (
        <>
        <div className="flex-1 p-6 space-x-6 flex">
            <StatusCard pipeline={PIPELINES[0]}/>
            <StatusCard pipeline={PIPELINES[1]}/>
            <StatusCard pipeline={PIPELINES[2]}/>
        </div>
        <PipelineChart/>
        </>
    )
}