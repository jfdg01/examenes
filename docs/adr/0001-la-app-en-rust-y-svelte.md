# La app de práctica corre en Rust y Svelte 5

La app de práctica es un servidor de Rust y una interfaz de Svelte 5. El código vive en `app/`. El objetivo es una app robusta, con un código fácil de leer y fácil de escalar a más exámenes. Dónde corre la lógica, y el formato de `respuestas.jsonl`, están en [el ADR 0002](0002-el-nucleo-en-wasm.md).

## Las decisiones

- **La lógica vive en Rust.** Un crate `nucleo` tiene toda la lógica: el paso de los conceptos, la sesión, el simulacro y la tarjeta de la nota. `nucleo` no lee archivos ni la red, y sus tests corren con `cargo test`.
- **La interfaz es Svelte 5 con Vite, sin SvelteKit.** La interfaz tiene una sola página. Vite compila Svelte a archivos estáticos en `dist/`, y el servidor de Rust los sirve.
- **Los componentes van en TypeScript.**
- **El servidor usa `axum`**, con `tokio` y `tower-http`.
- **Un solo proceso sirve todos los exámenes.** Cada examen tiene su ruta: `/ln/`, `/asn/` y `/ps/`. Un examen nuevo es solo una carpeta nueva, sin un puerto nuevo ni una ruta nueva de `tailscale serve`. Todos los exámenes comparten un origen, así que cada clave de `localStorage` lleva el nombre del examen.
- **Las herramientas del banco también van en Rust.** El proyecto no tiene Python.
- **El formato de los datos de un examen no cambia**, salvo `respuestas.jsonl`: `examen.json`, `preguntas.json`, `notas.json` y `ronda.json`.

## Las alternativas

- **La lógica en TypeScript**, con Rust solo para guardar los archivos. El dueño no quiere mantener la lógica en JavaScript ni en TypeScript.
- **SvelteKit con `adapter-static`.** Da el mismo resultado que Vite, con rutas y configuración que una sola página no usa.
- **Un proceso por examen**, con un puerto por examen. Cada examen nuevo necesita un puerto libre y una ruta de `tailscale serve`.
