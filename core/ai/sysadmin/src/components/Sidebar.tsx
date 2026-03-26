import React from 'react';
import { Page } from '../App';

interface SidebarProps {
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ currentPage, onPageChange }) => {
  const menuItems = [
    { id: 'dashboard' as Page, label: 'Dashboard', icon: '🏠' },
    { id: 'monitor' as Page, label: 'Monitor', icon: '📊' },
    { id: 'hardware' as Page, label: 'Hardware', icon: '⚡' },
    { id: 'ai' as Page, label: 'AI Insights', icon: '🧠' },
    { id: 'settings' as Page, label: 'Settings', icon: '⚙️' }
  ];

  return (
    <div className="w-64 bg-gray-800 h-screen flex flex-col">
      <div className="p-4 border-b border-gray-700">
        <h1 className="text-xl font-bold text-blue-400">AI SysAdmin</h1>
        <p className="text-sm text-gray-400">Control Center</p>
      </div>
      <nav className="flex-1 p-4">
        {menuItems.map(item => (
          <button
            key={item.id}
            onClick={() => onPageChange(item.id)}
            className={`w-full flex items-center space-x-3 p-3 rounded-lg mb-2 transition-colors ${
              currentPage === item.id
                ? 'bg-blue-600 text-white'
                : 'text-gray-300 hover:bg-gray-700'
            }`}
          >
            <span className="text-2xl">{item.icon}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
};
