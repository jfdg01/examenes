# La app de práctica corre en Rust y Svelte 5, con la lógica en el servidor

La app de práctica es un servidor de Rust y una interfaz de Svelte 5. El código vive en `app/`. El objetivo es una app robusta, con un código fácil de leer y fácil de escalar a más exámenes.

## Las decisiones

- **La lógica vive en Rust, en el servidor.** Un crate `nucleo` tiene toda la lógica: el paso de los conceptos, la sesión, el simulacro y la tarjeta de la nota. `nucleo` no lee archivos ni la red, y sus tests corren con `cargo test`. El servidor calcula el estado, y la interfaz lo muestra.
- **La interfaz es Svelte 5 con Vite, sin SvelteKit.** La interfaz tiene una sola página. Vite compila Svelte a archivos estáticos en `dist/`, y el servidor de Rust los sirve.
- **Los componentes van en TypeScript.** Los tipos del API salen de los tipos de Rust con `ts-rs`. Un cambio de un tipo en Rust rompe la build de la interfaz, y no la app en el móvil.
- **El servidor usa `axum`**, con `tokio` y `tower-http`.
- **Un solo proceso sirve todos los exámenes.** Cada examen tiene su ruta: `/ln/`, `/asn/` y `/ps/`. Un examen nuevo es solo una carpeta nueva, sin un puerto nuevo ni una ruta nueva de `tailscale serve`. Todos los exámenes comparten un origen, así que cada clave de `localStorage` lleva el nombre del examen.
- **Las herramientas del banco también van en Rust.** El proyecto no tiene Python.
- **El formato de los datos de un examen no cambia**: `examen.json`, `preguntas.json`, `respuestas.jsonl`, `notas.json` y `ronda.json`. El historial pasa a la versión de Rust sin una conversión.

## Las alternativas

- **La lógica en Rust compilada a WASM**, en el servidor y en el navegador. La interfaz calcularía sin el servidor, pero cada build necesita `wasm-pack`. Sin el servidor la página tampoco carga, así que el cálculo en el navegador no aporta nada.
- **La lógica en TypeScript**, con Rust solo para guardar los archivos. El dueño no quiere mantener la lógica en JavaScript ni en TypeScript.
- **SvelteKit con `adapter-static`.** Da el mismo resultado que Vite, con rutas y configuración que una sola página no usa.
- **Un proceso por examen**, con un puerto por examen. Cada examen nuevo necesita un puerto libre y una ruta de `tailscale serve`.
