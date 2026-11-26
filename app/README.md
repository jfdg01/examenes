# La app de práctica

Un servidor de Rust y una interfaz de Svelte 5. Las decisiones del stack están en [el ADR 0001](../docs/adr/0001-la-app-en-rust-y-svelte.md), y dónde corre la lógica está en [el ADR 0002](../docs/adr/0002-el-nucleo-en-wasm.md). Los diagramas están en PlantUML, en [`docs/diagramas/`](../docs/diagramas/README.md), con el modelo C4.

- `nucleo/`: toda la lógica, sin leer archivos ni la red. Corre en el navegador como WASM, y el servidor usa sus tipos para validar.
- `servidor/`: el servidor de `axum`. Guarda y sirve los archivos de cada examen, con gzip.
- `banco/`: las herramientas del banco de un examen, para su dueño: `banco comprobar`, `banco importar`, `banco montar`, `banco paginas`, `banco preparar` y `banco revision`, con las reglas de [`BANCO.md`](../docs/BANCO.md). `comprobar` necesita `pdfinfo`, de poppler. `paginas` necesita `pdfinfo`, `pdftoppm` y `pdftotext`, de poppler, y `ffmpeg`, con `libwebp`.
- `interfaz/`: la interfaz de Svelte 5, con Vite, TypeScript y Tailwind 4. `main.ts` monta `Lista.svelte` en `/`, `Plan.svelte` en `/?plan` y `App.svelte` en `/<carpeta>/`. Cada pantalla lleva en su línea de arriba `Pomodoro.svelte`, el botón del cronómetro de pausas, menos en un simulacro.

## Arrancar

Los comandos de la instalación y del arranque están en [el README de la raíz](../README.md). `wasm-bindgen-cli` lleva la versión de `wasm-bindgen` de `nucleo/Cargo.toml`.

El servidor recibe la raíz de los exámenes y su dirección. Cada carpeta de la raíz con `examen.json` es un examen en `/<carpeta>/`. El servidor lee la raíz en cada petición, así que un examen nuevo aparece sin reiniciar.

`npm run build` corre antes `npm run wasm`, que deja el módulo de `nucleo` y sus tipos de `tsify` en `interfaz/src/nucleo/`, fuera de git. Un cambio de un tipo en Rust rompe la build de la interfaz.

## Los tests

`cargo test` corre los tests de `nucleo`, del servidor y de `banco`. Cada test del servidor crea una raíz temporal de exámenes y le habla por HTTP.

El hook `.githooks/pre-commit` corre `cargo test` en un commit que cambia `app/`, y `banco comprobar` en un commit que cambia el banco de un examen. Un clon nuevo lo activa con `git config core.hooksPath .githooks`.

`npm run comprobar`, después de `npm run build`, corre en Playwright cada pantalla en el ancho del móvil y en el del PC, contra una copia de `examenes/ln/`. Las capturas quedan en `interfaz/capturas/`, fuera de git. Las clases sin estilo, como `examen`, `opcion` o `tirador`, existen para las aserciones de la comprobación: no las borres.
