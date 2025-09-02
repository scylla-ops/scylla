import {Button, Input, Label} from "@shadcn";
import {useState} from "react";
import * as React from "react";

type LoginFormProps = {
    handleSubmit: (e: React.FormEvent) => void;
}

export const LoginForm = ( {handleSubmit} : LoginFormProps) => {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");

    return (
        <form onSubmit={handleSubmit} className="flex flex-col gap-4 w-auto mx-auto">
            <div>
                <Label htmlFor="email">Email</Label>
                <Input
                    id="email"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="you@example.com"
                    required
                />
            </div>

            <div>
                <Label htmlFor="password">Password</Label>
                <Input
                    id="password"
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="••••••••"
                    required
                />
            </div>

            <Button type="submit" className="mt-2">
                Login
            </Button>
        </form>
    );
}