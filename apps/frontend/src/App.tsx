import { RouterProvider } from 'react-router-dom';
import { coreRouter } from '@core/presentation/ui/CoreRouter.tsx';

function App() {
  return <RouterProvider router={coreRouter} />;
}

export default App;
