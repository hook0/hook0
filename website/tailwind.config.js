const colors = require('tailwindcss/colors');

// Remove deprecated Tailwind CSS colors
delete colors.lightBlue;
delete colors.warmGray;
delete colors.trueGray;
delete colors.coolGray;
delete colors.blueGray;

module.exports = {
  content: ['src/**/*.ejs', 'src/**/*.svg'],
  theme: {
    extend: {
      // Typography Scale (Stripe-inspired fluid typography)
      fontSize: {
        'display-xl': ['clamp(2.5rem, 5vw + 1rem, 4.5rem)', { lineHeight: '1.1', letterSpacing: '-0.02em', fontWeight: '800' }],
        'display-lg': ['clamp(2rem, 4vw + 0.5rem, 3.5rem)', { lineHeight: '1.15', letterSpacing: '-0.02em', fontWeight: '800' }],
        'display': ['clamp(1.75rem, 3vw + 0.5rem, 2.5rem)', { lineHeight: '1.2', letterSpacing: '-0.01em', fontWeight: '700' }],
        'heading-xl': ['clamp(1.5rem, 2vw + 0.5rem, 2rem)', { lineHeight: '1.25', fontWeight: '700' }],
        'heading-lg': ['clamp(1.25rem, 1.5vw + 0.5rem, 1.5rem)', { lineHeight: '1.3', fontWeight: '600' }],
        'heading': ['clamp(1.125rem, 1.25vw + 0.5rem, 1.25rem)', { lineHeight: '1.4', fontWeight: '600' }],
        'body-lg': ['1.125rem', { lineHeight: '1.75' }],
        'body': ['1rem', { lineHeight: '1.75' }],
        'body-sm': ['0.875rem', { lineHeight: '1.6' }],
      },

      // Spacing System (generous vertical rhythm)
      spacing: {
        '18': '4.5rem',
        '22': '5.5rem',
        '26': '6.5rem',
        '30': '7.5rem',
        '34': '8.5rem',
        '38': '9.5rem',
        '42': '10.5rem',
        '50': '12.5rem',
        '60': '15rem',
        '72': '18rem',
        '84': '21rem',
        '96': '24rem',
      },

      // Colors (enhanced with surface colors)
      colors: {
        ...colors,
        surface: {
          primary: '#0a0a0f',
          secondary: '#111118',
          tertiary: '#1a1a24',
          elevated: '#242432',
        },
        brand: {
          indigo: colors.indigo,
          green: colors.green,
        },
      },

      // Animation keyframes
      keyframes: {
        'fade-in-up': {
          '0%': { opacity: '0', transform: 'translateY(30px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'slide-in-left': {
          '0%': { opacity: '0', transform: 'translateX(-30px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        'slide-in-right': {
          '0%': { opacity: '0', transform: 'translateX(30px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        'scale-in': {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        'float': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        'pulse-glow': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(99, 102, 241, 0.3)' },
          '50%': { boxShadow: '0 0 40px rgba(99, 102, 241, 0.5)' },
        },
        'gradient-shift': {
          '0%, 100%': { backgroundPosition: '0% 50%' },
          '50%': { backgroundPosition: '100% 50%' },
        },
      },

      // Animation utilities
      animation: {
        'fade-in-up': 'fade-in-up 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'fade-in': 'fade-in 0.5s ease-out forwards',
        'slide-in-left': 'slide-in-left 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'slide-in-right': 'slide-in-right 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'scale-in': 'scale-in 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'float': 'float 6s ease-in-out infinite',
        'pulse-glow': 'pulse-glow 3s ease-in-out infinite',
        'gradient-shift': 'gradient-shift 8s ease infinite',
      },

      // Transition timing (smooth transitions)
      transitionTimingFunction: {
        'smooth': 'cubic-bezier(0.16, 1, 0.3, 1)',
        'bounce-soft': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },

      // Box shadows (configurable card shadow levels)
      boxShadow: {
        'card-sm': '0 1px 2px rgba(0, 0, 0, 0.05)',
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        'card-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        'card-xl': '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
        'card-hover': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
        'glow-indigo': '0 0 40px rgba(99, 102, 241, 0.3)',
        'glow-green': '0 0 40px rgba(34, 197, 94, 0.3)',
        'glow-indigo-lg': '0 0 60px rgba(99, 102, 241, 0.4)',
        'inner-glow': 'inset 0 0 30px rgba(255, 255, 255, 0.05)',
      },

      // Border radius
      borderRadius: {
        '4xl': '2rem',
        '5xl': '2.5rem',
      },

      // Background images for gradients
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
        'mesh-gradient': 'linear-gradient(135deg, rgba(99,102,241,0.1) 0%, transparent 50%, rgba(34,197,94,0.1) 100%)',
      },

      // Font family
      fontFamily: {
        'sans': ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        'display': ['Inter', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Fira Code', 'ui-monospace', 'monospace'],
      },

      // Max width
      maxWidth: {
        '8xl': '88rem',
      },
    },
  },
  plugins: [
    require('postcss-import'),
    require('tailwindcss'),
    require('postcss-100vh-fix'),
    require('@tailwindcss/typography'),
  ],
};
