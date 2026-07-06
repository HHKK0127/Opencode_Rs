/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#3b82f6',
          dark: '#1e40af',
          light: '#dbeafe',
        },
      },
      fontFamily: {
        sans: ['Geist Sans', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
