import {createContext, useContext} from "react";
import * as React from "react";

class Dependencies {
 //TODO: add prod dependencies
}

const DependenciesContext = createContext<Dependencies | null>(null)

export const DependenciesProvider: React.FC<{ children: React.ReactNode}> = ({ children }) => {
    const dependencies = useContext(DependenciesContext)
    return <DependenciesContext.Provider value={dependencies}>{children}</DependenciesContext.Provider>
}