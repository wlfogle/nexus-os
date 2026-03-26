import { NavLink } from 'react-router-dom';
import {
  HomeIcon,
  ComputerDesktopIcon,
  CircleStackIcon,
  GlobeAltIcon,
  CogIcon,
} from '@heroicons/react/24/outline';

const navigation = [
  { name: 'Dashboard', href: '/', icon: HomeIcon },
  { name: 'Virtual Machines', href: '/virtual-machines', icon: ComputerDesktopIcon },
  { name: 'Storage', href: '/storage', icon: CircleStackIcon },
  { name: 'Networks', href: '/networks', icon: GlobeAltIcon },
  { name: 'Settings', href: '/settings', icon: CogIcon },
];

export default function Sidebar() {
  return (
    <div className="flex flex-col w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
      {/* Logo */}
      <div className="flex items-center h-16 px-6 border-b border-gray-200 dark:border-gray-700">
        <ComputerDesktopIcon className="w-8 h-8 text-blue-600" />
        <span className="ml-2 text-xl font-bold text-gray-900 dark:text-white">
          KVM Manager
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-1">
        {navigation.map((item) => (
          <NavLink
            key={item.name}
            to={item.href}
            className={({ isActive }) =>
              `group flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                isActive
                  ? 'bg-blue-50 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 border-r-2 border-blue-600'
                  : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 hover:text-gray-900 dark:hover:text-white'
              }`
            }
          >
            <item.icon
              className="w-5 h-5 mr-3 text-gray-400 group-hover:text-gray-500 dark:group-hover:text-gray-300"
              aria-hidden="true"
            />
            {item.name}
          </NavLink>
        ))}
      </nav>

      {/* Version info */}
      <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700">
        <div className="text-xs text-gray-500 dark:text-gray-400">
          <div>KVM Manager v0.1.0</div>
          <div>Built with Tauri + React</div>
        </div>
      </div>
    </div>
  );
}
