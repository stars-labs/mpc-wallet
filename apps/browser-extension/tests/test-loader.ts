// Loader to handle #imports in test environment
import { plugin } from "bun";

plugin({
  name: "imports-resolver",
  setup(build) {
    // Intercept imports of #imports
    build.onResolve({ filter: /^#imports$/ }, args => {
      return {
        path: require.resolve("./tests/__mocks__/imports.ts"),
        namespace: "imports"
      };
    });
  },
});