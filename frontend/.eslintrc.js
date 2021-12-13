module.exports = {
  root: true,

  env: {
    node: true,
  },

  parser: 'vue-eslint-parser',

  parserOptions: {
    ecmaVersion: 2021,
    parser: '@typescript-eslint/parser',
    tsconfigRootDir: __dirname,
    project: ['./tsconfig.json'],
    sourceType: 'module',
    extraFileExtensions: ['.vue'],
  },

  plugins: ['@typescript-eslint'],

  rules: {
    'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'comma-style': ['error', 'last'],
    'vue/no-multiple-template-root': 'off',
    '@typescript-eslint/ban-ts-comment': 'off',
    'vue/multi-word-component-names': [
      'error',
      {
        ignores: ['Promised', 'Error404'],
      },
    ],
    'no-unused-vars': 'off', // Disabled because overseeded by the @typescript-eslint/no-unused-vars rule
    '@typescript-eslint/no-unused-vars': [
      'warn',
      {
        varsIgnorePattern: '^[_].*',
        argsIgnorePattern: '^[_].*',
      }
    ],
  },

  extends: [
    'plugin:vue/vue3-essential',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    // 'plugin:prettier-vue/recommended', // TODO: find a way to make it work
    'prettier',
  ],

  overrides: [
    { // As types.ts is generated, we have to disable rules that it violates
      files: ['src/types.ts'],
      rules: {
        '@typescript-eslint/no-empty-interface': 'off',
      },
    },
    {
      files: ['.eslintrc.js'],
      rules: {
        '@typescript-eslint/no-unsafe-assignment': 'off',
      },
    },
  ]
};
