module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.json',
    ecmaVersion: 'es2019',
    sourceType: 'module',
    tsconfigRootDir: __dirname,
    ecmaFeatures: {
      legacyDecorators: true,
    },
  },
  "extends": [
    '../.eslintrc.js',
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    'plugin:import/typescript',
    "plugin:mocha/recommended",
    "prettier"
  ],
  plugins: ['mocha'],
  ignorePatterns: ['.eslintrc.js'],
};
