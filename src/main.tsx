import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

// NUCLEAR SCROLLBAR FORCING - RUNTIME INJECTION
const forceScrollbars = () => {
  const style = document.createElement('style');
  style.id = 'force-scrollbars';
  style.innerHTML = `
    * {
      scrollbar-width: auto !important;
      scrollbar-color: #6B7280 #374151 !important;
    }
    *::-webkit-scrollbar {
      width: 16px !important;
      height: 16px !important;
      display: block !important;
      background: #374151 !important;
    }
    *::-webkit-scrollbar-track {
      background: #374151 !important;
      border-radius: 8px !important;
    }
    *::-webkit-scrollbar-thumb {
      background: #6B7280 !important;
      border-radius: 8px !important;
      border: 2px solid #374151 !important;
    }
    *::-webkit-scrollbar-thumb:hover {
      background: #9CA3AF !important;
    }
    .overflow-hidden { overflow: auto !important; }
    .flex-1 { overflow: auto !important; }
    .h-full { overflow: auto !important; }
  `;
  document.head.insertBefore(style, document.head.firstChild);
  
  // Also force DOM elements directly
  const observer = new MutationObserver(() => {
    document.querySelectorAll('*').forEach(el => {
      if (el instanceof HTMLElement) {
        const computedStyle = window.getComputedStyle(el);
        if (computedStyle.overflow === 'hidden' && 
            (el.scrollHeight > el.clientHeight || el.scrollWidth > el.clientWidth)) {
          el.style.overflow = 'auto';
          el.style.overflowY = 'scroll';
        }
      }
    });
  });
  observer.observe(document.body, { childList: true, subtree: true });
};

// Run immediately and on DOM ready
forceScrollbars();
document.addEventListener('DOMContentLoaded', forceScrollbars);
window.addEventListener('load', forceScrollbars);

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
