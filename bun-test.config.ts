// Bun test configuration
export default {
  preload: ['./tests/setup-bun.ts'],
  define: {
    'import.meta.env.DEV': 'false',
    'import.meta.env.PROD': 'true'
  },
  alias: {
    '#imports': './tests/__mocks__/imports.ts'
  }
};