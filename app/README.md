# La app de práctica

Un servidor de Rust y una interfaz de Svelte 5. Las decisiones del stack están en [el ADR 0001](../docs/adr/0001-la-app-en-rust-y-svelte.md). Los diagramas de las clases, la ronda y una sesión están en PlantUML, en [el diagrama](../docs/el-diagrama.puml).

- `nucleo/`: toda la lógica, sin leer archivos ni la red.
- `servidor/`: el servidor de `axum`. Lee la carpeta de cada examen y llama a `nucleo`.
- `interfaz/`: la interfaz de Svelte 5, con Vite y TypeScript.

## Arrancar

```bash
cd app/interfaz && npm install && npm run build
cd .. && cargo run -p servidor -- .. 127.0.0.1:8601
```

El servidor recibe la raíz de los exámenes y su dirección. Sirve cada carpeta de la raíz con `examen.json` en `/<carpeta>/`, y lee la carpeta en cada petición. Un examen nuevo aparece al crear su carpeta, sin reiniciar el servidor.

## Los tipos del API

Los tipos del API salen de `nucleo` con `ts-rs`, en `interfaz/src/tipos/`, que no entra en git. `npm run build` genera los tipos antes de `svelte-check`, así que un cambio de un tipo en Rust rompe la build de la interfaz. `cargo test` también los genera.

## Los tests

`cargo test` corre los tests de `nucleo` y del servidor. Cada test del servidor crea una raíz temporal de exámenes, arranca el servidor y le habla por HTTP.

En desarrollo, `npm run dev` sirve la interfaz con Vite y pasa el API al servidor de `127.0.0.1:8601`.
