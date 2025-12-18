module.exports = {
  // paths to all of your pages and components so Tailwind can tree-shake unused styles in production build
  content: ['./src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          primary: '#0a0a0f',
          secondary: '#111118',
          tertiary: '#1a1a24',
          elevated: '#242432',
        },
      },
      boxShadow: {
        'glow-indigo': '0 0 40px rgba(99, 102, 241, 0.3)',
        'glow-green': '0 0 40px rgba(34, 197, 94, 0.3)',
      },
      transitionTimingFunction: {
        smooth: 'cubic-bezier(0.16, 1, 0.3, 1)',
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [require('@tailwindcss/forms')],
};
