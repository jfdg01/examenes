import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// En desarrollo, Vite pasa al servidor de Rust la lista y cada ruta de un examen.
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  // PROTOTIPO: allowedHosts deja pasar la tailnet.
  server: { allowedHosts: [".ts.net"], proxy: { "^/api/|^/[^/]+/(api|paginas|pdfs)/": "http://127.0.0.1:8602" } },
});
