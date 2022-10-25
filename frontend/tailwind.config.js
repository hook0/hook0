module.exports = {
  // paths to all of your pages and components so Tailwind can tree-shake unused styles in production build
  content: ['./src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [require('@tailwindcss/forms')],
};
