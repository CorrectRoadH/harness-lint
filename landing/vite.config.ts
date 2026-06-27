import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Relative base so the static build works from any path (custom domain,
// GitHub Pages project subpath, or a plain file server).
export default defineConfig({
  base: "./",
  plugins: [react()],
});
