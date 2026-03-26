/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Terminal colors
        terminal: {
          bg: '#1a1a1a',
          fg: '#ffffff',
          black: '#000000',
          red: '#ff5555',
          green: '#50fa7b',
          yellow: '#f1fa8c',
          blue: '#bd93f9',
          magenta: '#ff79c6',
          cyan: '#8be9fd',
          white: '#bfbfbf',
          'bright-black': '#4d4d4d',
          'bright-red': '#ff6e6e',
          'bright-green': '#69ff94',
          'bright-yellow': '#ffffa5',
          'bright-blue': '#d6acff',
          'bright-magenta': '#ff92df',
          'bright-cyan': '#a4ffff',
          'bright-white': '#ffffff',
        },
        // Custom grays
        gray: {
          850: '#1e1e1e',
          750: '#2a2a2a',
        }
      },
      fontFamily: {
        'mono': ['JetBrains Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', 'monospace'],
        'sans': ['Inter', 'system-ui', 'sans-serif'],
      },
      fontSize: {
        'terminal': ['14px', { lineHeight: '1.2' }],
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(4px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-in-right': {
          '0%': { transform: 'translateX(100%)' },
          '100%': { transform: 'translateX(0)' },
        },
        'pulse-slow': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },
        'blink': {
          '0%, 50%': { opacity: '1' },
          '51%, 100%': { opacity: '0' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.2s ease-out',
        'slide-in-right': 'slide-in-right 0.3s ease-out',
        'pulse-slow': 'pulse-slow 2s infinite',
        'blink': 'blink 1s step-end infinite',
      },
      backdropBlur: {
        'xs': '2px',
      },
    },
  },
  plugins: [
    require('tailwindcss-scrollbar'),
    // FORCE SCROLLBARS EVERYWHERE
    function({ addBase, addUtilities }) {
      addBase({
        '*': {
          'scrollbar-width': 'auto !important',
          'scrollbar-color': '#6B7280 #374151 !important',
        },
        '*::-webkit-scrollbar': {
          'width': '16px !important',
          'height': '16px !important',
          'display': 'block !important',
          'background': '#374151 !important',
        },
        '*::-webkit-scrollbar-track': {
          'background': '#374151 !important',
          'border-radius': '8px !important',
        },
        '*::-webkit-scrollbar-thumb': {
          'background': '#6B7280 !important',
          'border-radius': '8px !important',
          'border': '2px solid #374151 !important',
        },
        '*::-webkit-scrollbar-thumb:hover': {
          'background': '#9CA3AF !important',
        },
        '.overflow-hidden': {
          'overflow': 'auto !important',
        },
      })
    }
  ],
}
