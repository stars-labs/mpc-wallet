// Preload script for Bun tests
// This handles module resolution for #imports

import { plugin } from 'bun';

plugin({
  name: 'imports-resolver',
  setup(build) {
    // Intercept #imports and redirect to mock
    build.onResolve({ filter: /^#imports$/ }, (args) => {
      return {
        path: require.resolve('./tests/__mocks__/imports.ts'),
        namespace: 'imports-mock'
      };
    });
    
    build.onLoad({ filter: /.*/, namespace: 'imports-mock' }, async (args) => {
      return {
        contents: `
          export const browser = globalThis.chrome || {
            runtime: {
              id: 'test-extension-id',
              sendMessage: () => {},
              onMessage: {
                addListener: () => {},
                removeListener: () => {}
              }
            },
            storage: {
              local: {
                get: () => Promise.resolve({}),
                set: () => Promise.resolve(),
                remove: () => Promise.resolve()
              }
            }
          };
          
          export default { browser };
        `,
        loader: 'ts'
      };
    });
  }
});