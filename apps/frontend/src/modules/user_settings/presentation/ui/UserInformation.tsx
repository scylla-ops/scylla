import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    Avatar,
    AvatarFallback,
    AvatarImage, Button
} from '@/modules/core/presentation/ui/shadcn';


export const UserInformation = () => {
    return (

        <Card className="w-full bg-white">

            <CardHeader>
                <CardTitle>User information</CardTitle>
                <CardDescription>Manage your account details.</CardDescription>
            </CardHeader>

            <CardContent className="space-y-4">

                <div className="flex items-center space-x-4">
                    <Avatar>
                        <AvatarImage src="https://github.com/YohannMgt.png" />
                        <AvatarFallback>YM</AvatarFallback>
                    </Avatar>
                    <div>
                        <div className="text-base font-medium">Yohann Mangenot</div>
                        <div className="text-sm text-gray-500">Cloud plan</div>
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">First name</label>
                    <input
                        type="text"
                        value="Yohann"
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    />
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">Last name</label>
                    <input
                        type="text"
                        value="Mangenot"
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    />
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">Language</label>
                    <select
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    >
                        <option>English (US)</option>
                    </select>
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">Email</label>
                    <input
                        type="email"
                        value="yohann.mangenot@epitech.eu"
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    />
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">Phone</label>
                    <input
                        type="tel"
                        value="+33 1 22 33 44 55"
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600  shadow-sm p-2"
                        disabled
                    />
                </div>

                <div>
                    <label className="block text-sm font-bold text-black">Password</label>
                    <input
                        type="password"
                        value="password"
                        className="mt-1 block w-full rounded-md border border-gray-300 text-gray-600 shadow-sm p-2"
                        disabled
                    />
                </div>

                <Button className="w-16 float-right">Edit</Button>

            </CardContent>

        </Card>

    );
};
