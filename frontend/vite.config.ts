import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    watch: {
      usePolling: true,
    },
    fs: {
      allow: [
        "..", // 👈 allow parent folder (simplest)
      ],
    },
  },
  build: {
    target: "esnext",
  },
});
