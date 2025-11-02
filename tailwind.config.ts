import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{html,ts}'],
  theme: {
    extend: {
      colors: {
        'itunes-bg': '#000000',
        'itunes-sidebar': '#1a1a1a',
        'itunes-hover': '#2a2a2a',
        'itunes-selected': '#3a3a3a',
        'itunes-accent': '#fc3c44',
        'itunes-text': '#ffffff',
        'itunes-text-secondary': '#999999',
      },
      fontFamily: {
        'sans': ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
      },
    },
  },
  plugins: [],
} satisfies Config;

