/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // AMOLED Black Theme
        'amoled': {
          'black': '#000000',
          'dark': '#0a0a0a',
          'darker': '#151515',
          'gray': '#1a1a1a',
          'light-gray': '#2a2a2a',
          'border': '#333333',
          'text': '#ffffff',
          'text-secondary': '#b3b3b3',
          'text-muted': '#666666',
          'accent': '#00d4ff',
          'accent-hover': '#00b8e6',
          'success': '#00ff88',
          'warning': '#ffaa00',
          'error': '#ff4444',
        }
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
      fontFamily: {
        'mono': ['JetBrains Mono', 'Consolas', 'Monaco', 'monospace'],
      },
      boxShadow: {
        'amoled': '0 4px 20px rgba(0, 212, 255, 0.1)',
        'amoled-lg': '0 10px 40px rgba(0, 212, 255, 0.15)',
      }
    },
  },
  plugins: [],
}