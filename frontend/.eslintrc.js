module.exports = {
  root: true,

  env: {
    node: true,
  },

  parser: 'vue-eslint-parser',

  parserOptions: {
    ecmaVersion: 2021,
    parser: '@typescript-eslint/parser',
    // tsconfigRootDir: __dirname,
    // project: ['./tsconfig.json'],
    // sourceType: 'module',
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
  },

  extends: [
    'plugin:vue/vue3-essential',
    'plugin:@typescript-eslint/recommended',
    // 'plugin:@typescript-eslint/recommended-requiring-type-checking',
    // 'plugin:prettier-vue/recommended', // TODO: find a way to make it work
    'prettier',
  ],
};
