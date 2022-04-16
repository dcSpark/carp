module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.typecheck.json',
    ecmaVersion: 'es2019',
    sourceType: 'module',
    tsconfigRootDir: __dirname,
    ecmaFeatures: {
      legacyDecorators: true,
    },
  },
  extends: [
    '../.eslintrc.js',
  ],
  ignorePatterns: ['.eslintrc.js'],
};
