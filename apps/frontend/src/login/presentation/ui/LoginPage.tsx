import { LoginForm } from "@/login/presentation/ui/LoginForm.tsx";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shadcn";
import LogoScylla from "@/assets/logo_scylla.png"

export const LoginPage = () => {
    return (
        <div className={"flex items-center h-screen flex-col"}>
            <div className="flex flex-row items-center space-x-1 mb-4 mt-12 pr-6">
                <img src={LogoScylla} alt="logo" className="object-contain w-32 h-32" />
                <h1 className="text-4xl font-bold">Scylla</h1>
            </div>
            <Card className="w-full max-w-sm">
                <CardHeader>
                    <CardTitle>Login to your account</CardTitle>
                    <CardDescription>
                        Enter your email below to login to your account
                    </CardDescription>
                </CardHeader>
                <CardContent>
                   <LoginForm handleSubmit={(e => e.preventDefault())}/>
                </CardContent>
            </Card>
        </div>
    )
}

export default LoginPage;