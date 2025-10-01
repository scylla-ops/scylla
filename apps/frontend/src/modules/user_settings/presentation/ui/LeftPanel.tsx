import { useState } from 'react';

export const LeftPanel = () => {
    const [selected, setSelected] = useState("settings");

    const menuItems = [
        { id: "dashboard", label: "Dashboard" },
        { id: "repositories", label: "Repositories" },
        { id: "plugins", label: "Plugins marketplace" },
    ];

    return (
        <div className="flex flex-col w-64 h-screen border-r bg-gray-50">

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
                                {item.label}
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
                    Settings
                </button>
                <button
                    onClick={() => setSelected("account")}
                    className={`w-full text-left px-3 py-2 rounded-lg transition ${
                        selected === "account"
                            ? "bg-gray-200 text-gray-900 font-medium"
                            : "text-gray-700 hover:bg-gray-100"
                    }`}
                >
                    Select an account
                </button>
            </div>
        </div>
    );
}
