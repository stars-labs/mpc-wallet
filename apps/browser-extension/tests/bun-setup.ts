import { plugin } from 'bun';

// Register a custom loader for #imports
plugin({
  name: 'imports-resolver',
  setup(build) {
    build.onResolve({ filter: /^#imports$/ }, () => {
      return {
        path: './tests/__mocks__/#imports.ts',
        namespace: 'imports-mock'
      };
    });
    
    build.onLoad({ filter: /.*/, namespace: 'imports-mock' }, () => {
      return {
        contents: `
          export const storage = {
            getItem: async () => null,
            setItem: async () => undefined,
            removeItem: async () => undefined,
            clear: async () => undefined,
            defineItem: (key, options) => ({
              getValue: async () => options?.fallback ?? null,
              setValue: async () => undefined,
              removeValue: async () => undefined,
              key,
              options
            })
          };
          
          export const browser = {
            runtime: {
              sendMessage: () => Promise.resolve(),
              onMessage: {
                addListener: () => {},
                removeListener: () => {}
              },
              getURL: (path) => \`chrome-extension://mock-extension-id/\${path}\`,
              id: 'mock-extension-id'
            },
            storage: {
              local: {
                get: () => Promise.resolve({}),
                set: () => Promise.resolve(),
                remove: () => Promise.resolve(),
                clear: () => Promise.resolve()
              }
            }
          };
        `,
        loader: 'ts'
      };
    });
  }
});