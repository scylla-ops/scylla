import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
} from '@/modules/core/presentation/ui/shadcn';

export const Organizations = () => {
    return (

        <Card className="w-full">

            <CardHeader>
                <CardTitle>Organizations</CardTitle>
                <CardDescription>Manage your organizations.</CardDescription>
            </CardHeader>

            <CardContent className="space-y-4">

                <div className="flex items-center justify-between rounded-md">
                    <div>
                        <div className="text-base font-medium">Alpha Organization</div>
                        <div className="text-sm text-gray-500">For 3 months</div>
                    </div>
                    <div>
                    <select
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    >
                        <option>Owner</option>
                    </select>
                    </div>
                </div>

                <div className="flex items-center justify-between rounded-md">
                    <div>
                        <div className="text-base font-medium">Beta Organization</div>
                        <div className="text-sm text-gray-500">For 2 years</div>
                    </div>
                    <div>
                        <select
                            className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                            disabled
                        >
                            <option>Member</option>
                        </select>
                    </div>
                </div>

                <div className="flex items-center justify-between rounded-md">
                    <div>
                        <div className="text-base font-medium">Gamma Organization</div>
                        <div className="text-sm text-gray-500">For 3 years</div>
                    </div>
                    <div>
                        <select
                            className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                            disabled
                        >
                            <option>Member</option>
                        </select>
                    </div>
                </div>

                <div className="flex items-center justify-between rounded-md">
                    <div>
                        <div className="text-base font-medium">Delta Organization</div>
                        <div className="text-sm text-gray-500">For 1 week</div>
                    </div>
                    <div>
                        <select
                            className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                            disabled
                        >
                            <option>Member</option>
                        </select>
                    </div>
                </div>

            </CardContent>

        </Card>

    );
};
