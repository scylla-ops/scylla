import { useState } from 'react';
import {House, Folders, Blocks, Settings} from 'lucide-react';
import {
    Avatar,
    AvatarFallback,
    AvatarImage
} from '@/modules/core/presentation/ui/shadcn';


export const LeftPanel = () => {
    const [selected, setSelected] = useState("settings");

    const menuItems = [
        { id: "dashboard", label: "Dashboard", icon: House },
        { id: "repositories", label: "Repositories", icon: Folders},
        { id: "plugins", label: "Plugins marketplace", icon: Blocks},
    ];

    return (
        <div className="flex flex-col h-[884px] border-r bg-[#f5f5f5]">

            {/* Main navigation */}
            <nav className="flex-1 px-2 py-4">
                <ul className="space-y-1">
                    {menuItems.map(item => (
                        <li key={item.id}>
                            <button
                                onClick={() => setSelected(item.id)}
                                className={`w-full text-left px-3 py-2 rounded-lg transition ${
                                    selected === item.id
                                        ? "bg-gray-200 text-gray-900 font-medium"
                                        : "text-gray-700 hover:bg-gray-100"
                                }`}
                            >
                                <span className="flex items-center">
                                    <item.icon className="w-5 h-5 mr-2" />
                                    {item.label}
                                </span>
                            </button>
                        </li>
                    ))}
                </ul>
            </nav>

            {/* Bottom section */}
            <div className="border-t px-2 py-3 space-y-1">
                <button
                    onClick={() => setSelected("settings")}
                    className={`w-full text-left px-3 py-2 rounded-lg transition ${
                        selected === "settings"
                            ? "bg-gray-200 text-gray-900 font-medium"
                            : "text-gray-700 hover:bg-gray-100"
                    }`}
                >
                    <span className="flex items-center">
                        <Settings className="w-5 h-5 mr-2" />
                        Settings
                    </span>
                </button>
                <button
                    onClick={() => setSelected("account")}
                    className={`w-full text-left px-3 py-2 rounded-lg transition ${
                        selected === "account"
                            ? "bg-gray-200 text-gray-900 font-medium"
                            : "text-gray-700 hover:bg-gray-100"
                    }`}
                >
                    <span className="flex items-center">
                        <Avatar>
                            <AvatarImage src="https://github.com/YohannMgt.png" />
                            <AvatarFallback>YM</AvatarFallback>
                        </Avatar>
                        <span className="ml-4">Select an account</span>
                    </span>

                </button>
            </div>
        </div>
    );
}
