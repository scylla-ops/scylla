import {BrowserRouter, Route, Navigate, Routes} from "react-router-dom";
import LoginPage from "@/login/presentation/ui/LoginPage.tsx";

//TODO: navigation and overlay
export const CoreRoot = () => {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<LoginPage/>} />
                <Route path="*" element={<Navigate to="/login" replace />} />
            </Routes>
        </BrowserRouter>
    )
}

export default CoreRoot;