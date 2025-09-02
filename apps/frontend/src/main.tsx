import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import {DependenciesProvider} from "@/core/presentation/providers/DependenciesProvider.tsx";

createRoot(document.getElementById('root')!).render(
  <StrictMode>
      <DependenciesProvider>
          <App />
      </DependenciesProvider>
  </StrictMode>,
)
