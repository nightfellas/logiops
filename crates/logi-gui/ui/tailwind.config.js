/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        logi: {
          dark: '#121316',
          panel: '#1e2025',
          card: '#272a31',
          border: '#383c46',
          teal: '#00d2c4',
          accent: '#2ecc71',
        },
      },
    },
  },
  plugins: [],
};
