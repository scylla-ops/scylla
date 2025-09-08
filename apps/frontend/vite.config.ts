import { defineConfig } from 'vite'
import react from "@vitejs/plugin-react-swc";
import tailwindcss from '@tailwindcss/vite'
import tsconfigPaths from 'vite-tsconfig-paths'
import lingui from "@lingui/vite-plugin";

export default defineConfig({
  plugins: [
      react({
          plugins: [["@lingui/swc-plugin", {}]]
      }),
      lingui(),
      tailwindcss(),
      tsconfigPaths()],
})
