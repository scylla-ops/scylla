import {Button} from "@core/presentation/ui/shadcn/button.tsx";
import {useState} from "react";


export const LoginPage = () => {
    const [count, setCount] = useState(0)

    return <>
        <Button onClick={ () => setCount(count + 1)}>Increment</Button>
        <p>Count : {count} </p>
    </>
}

export default LoginPage;