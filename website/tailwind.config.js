const colors = require('tailwindcss/colors');

module.exports = {
  purge: ["./**.html"],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
    colors: colors
  },
  variants: {
    extend: {},
  },
  plugins: [
    require("postcss-import"),
    require("tailwindcss"),
    require("postcss-100vh-fix"),
    require("@tailwindcss/typography"),
  ],
};
