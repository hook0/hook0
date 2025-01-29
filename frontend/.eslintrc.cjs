module.exports = {
  root: true,

  env: {
    node: true,
    es2022: true,
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
    'no-console': [process.env.NODE_ENV === 'production' ? 'warn' : 'off', { allow: ['error', 'warn'] }],
    'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'comma-style': ['error', 'last'],
    '@typescript-eslint/ban-ts-comment': 'off',
    'vue/multi-word-component-names': [
      'error',
      {
        ignores: ['Promised', 'Error404', 'Root', 'Home'],
      },
    ],
    'no-unused-vars': 'off', // Disabled because overseeded by the @typescript-eslint/no-unused-vars rule
    '@typescript-eslint/no-unused-vars': [
      'warn',
      {
        varsIgnorePattern: '^[_].*',
        argsIgnorePattern: '^[_].*',
      },
    ],
    'vue/component-api-style': ['error', ['script-setup']],
    'vue/component-name-in-template-casing': ['error', 'PascalCase'],
  },

  settings: {
    'prettier-vue': {
      // Settings for how to process Vue SFC Blocks
      SFCBlocks: {
        /**
         * Use prettier to process `<template>` blocks or not
         *
         * If set to `false`, you may need to enable those vue rules that are disabled by `eslint-config-prettier`,
         * because you need them to lint `<template>` blocks
         *
         * @default true
         */
        template: true,

        /**
         * Use prettier to process `<script>` blocks or not
         *
         * If set to `false`, you may need to enable those rules that are disabled by `eslint-config-prettier`,
         * because you need them to lint `<script>` blocks
         *
         * @default true
         */
        script: true,

        /**
         * Use prettier to process `<style>` blocks or not
         *
         * @default true
         */
        style: true,

        // Settings for how to process custom blocks
        customBlocks: {
          // Treat the `<docs>` block as a `.markdown` file
          docs: { lang: 'markdown' },

          // Treat the `<config>` block as a `.json` file
          config: { lang: 'json' },

          // Treat the `<module>` block as a `.js` file
          module: { lang: 'js' },

          // Ignore `<comments>` block (omit it or set it to `false` to ignore the block)
          comments: false,

          // Other custom blocks that are not listed here will be ignored
        },
      },

      // Use prettierrc for prettier options
      usePrettierrc: true,

      // Set the options for `prettier.getFileInfo`.
      // @see https://prettier.io/docs/en/api.html#prettiergetfileinfofilepath-options
      fileInfoOptions: {
        // Path to ignore file (default: `'.prettierignore'`)
        // Notice that the ignore file is only used for this plugin
        ignorePath: '.testignore',

        // Process the files in `node_modules` or not (default: `false`)
        withNodeModules: false,
      },
    },
  },

  extends: [
    'plugin:vue/vue3-essential',
    'plugin:vue/vue3-recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    'plugin:prettier-vue/recommended',
    'prettier',
  ],

  overrides: [
    {
      // As types.ts is generated, we have to disable rules that it violates
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
  ],
};
