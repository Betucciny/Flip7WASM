import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    watch: {
      usePolling: true,
    },
    fs: {
      allow: [".."],
    },
  },
  build: {
    target: "esnext",
  },
});
