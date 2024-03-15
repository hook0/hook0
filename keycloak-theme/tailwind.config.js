const colors = require('tailwindcss/colors')

module.exports = {
    purge: [
        "./src/**/*.scss"
    ],
    theme: {
        colors: {
            transparent: 'transparent',
            current: 'currentColor',
            black: colors.black,
            white: colors.white,
            gray: colors.gray,
            emerald: colors.emerald,
            indigo: colors.indigo,
            yellow: colors.yellow,
            ciam_dark: '#00318A',
            ciam_medium: '#3063BC',
            ciam_light: '#86E2FA'
        },
    },
    variants: {},
    plugins: [
        require('@tailwindcss/forms'),
    ]
};
