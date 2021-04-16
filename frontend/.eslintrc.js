module.exports = {
  root: true,

  env: {
    node: true,
  },

  extends: ['plugin:vue/vue3-essential', 'plugin:vue/recommended', '@vue/prettier'],

  parserOptions: {
    ecmaVersion: 2017,
    parser: '@typescript-eslint/parser',
  },

  rules: {
    'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'comma-style': ['error', 'last'],
    'vue/no-multiple-template-root': 'off',
  },

  extends: [
    'plugin:vue/vue3-essential',
    'plugin:vue/recommended',
    '@vue/prettier',
    '@vue/typescript',
  ],
};
