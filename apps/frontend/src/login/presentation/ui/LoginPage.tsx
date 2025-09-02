import {LoginForm} from "@/login/presentation/ui/LoginForm.tsx";
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from "@shadcn";

export const LoginPage = () => {
    return (
        <div className={"flex items-center justify-center h-screen"}>
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