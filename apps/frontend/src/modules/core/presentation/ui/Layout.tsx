import { Outlet } from 'react-router-dom';
import { Sidebar } from 'lucide-react';

export const Layout = () => (
  <div className='flex h-screen bg-gray-100'>
    <aside className='w-64 bg-white shadow-lg'>
      <Sidebar />
    </aside>
    <main className='flex-1 overflow-y-auto p-6'>
      <Outlet />
    </main>
  </div>
);
