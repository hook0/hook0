const colors = require('tailwindcss/colors');

// Remove Tailwind CSS warnings
delete colors.lightBlue;
delete colors.warmGray;
delete colors.trueGray;
delete colors.coolGray;
delete colors.blueGray;

module.exports = {
  content: ['./**.html', 'src/**.ejs'],
  theme: {
    extend: {},
    colors: colors,
  },
  plugins: [require('postcss-import'), require('tailwindcss'), require('postcss-100vh-fix'), require('@tailwindcss/typography')],
};
