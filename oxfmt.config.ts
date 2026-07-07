import { defineConfig } from 'oxfmt';

export default defineConfig({
  ignorePatterns: [
    '**/.github/**',
    '**/.vendor/**',
    '**/.kiro/**',
    '**/coverage/**',
    '**/dist/**',
    '**/node_modules/**',
    '**/k6-results/**',
  ],
  overrides: [
    {
      files: ['**/*.ts'],
      options: {
        singleQuote: true,
        printWidth: 120,
      },
    },
    {
      files: ['**/*.json', '**/*.jsonc'],
      options: {
        // Keep JSON compact for better readability of small config/data files
        printWidth: 20,
        trailingComma: 'none',
      },
    },
    {
      files: ['**/*.md'],
      options: {
        tabWidth: 4,
      },
    },
  ],
  sortImports: {
    newlinesBetween: false,
    groups: [
      'type-import',
      'type-internal',
      ['type-parent', 'type-sibling', 'type-index'],
      'value-builtin',
      'value-external',
      'value-internal',
      ['value-parent', 'value-sibling', 'value-index'],
      'unknown',
    ],
  },
});
