# La app de práctica

Un servidor de Rust y una interfaz de Svelte 5. Las decisiones del stack están en [el ADR 0001](../docs/adr/0001-la-app-en-rust-y-svelte.md), y dónde corre la lógica está en [el ADR 0002](../docs/adr/0002-el-nucleo-en-wasm.md). Los diagramas de las clases, la ronda y una sesión están en PlantUML, en [el diagrama](../docs/el-diagrama.puml).

- `nucleo/`: toda la lógica, sin leer archivos ni la red. Corre en el navegador como WASM, y el servidor usa sus tipos para validar.
- `servidor/`: el servidor de `axum`. Guarda y sirve los archivos de cada examen, con gzip.
- `interfaz/`: la interfaz de Svelte 5, con Vite y TypeScript. Llama a `nucleo` para cada cálculo.

## Arrancar

La build de la interfaz necesita el target `wasm32-unknown-unknown` y `wasm-bindgen-cli`, con la versión de `wasm-bindgen` de `nucleo/Cargo.toml`.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
```

```bash
cd app/interfaz && npm install && npm run build
cd .. && cargo run -p servidor -- .. 127.0.0.1:8601
```

El servidor recibe la raíz de los exámenes y su dirección. Sirve cada carpeta de la raíz con `examen.json` en `/<carpeta>/`, y lee la carpeta en cada petición. Un examen nuevo aparece al crear su carpeta, sin reiniciar el servidor.

- `GET api/inicio` da el examen, el banco y el historial.
- `POST api/respuestas` añade al historial cada respuesta que no está. Una línea de `respuestas.jsonl` lleva `id`, `ts`, `resultado` (`Acierto`, `Fallo` o `Blanco`) y `seguro`. La interfaz guarda cada respuesta en la clave `<carpeta>-pendientes` de `localStorage` hasta que llega al servidor. Al cargar la página y al volver la red, envía las que esperan.
- `GET api/ronda` da `ronda.json`, y `PUT api/ronda` escribe el guardado más nuevo entre el que llega y el de `ronda.json`: `{"guardada", "ronda"}`, con la hora del guardado en milisegundos y la ronda a medias o `null`. La interfaz guarda la misma ronda en la clave `<carpeta>-ronda` de `localStorage`, y al cargar envía el guardado más nuevo de los dos.
- `GET paginas/<archivo>` da la imagen de una página del PDF, y `GET pdfs/<archivo>` da un PDF. Los 2 leen la carpeta `paginas/` o `pdfs/` del examen, y dan 404 a un nombre con `/` o con un punto delante.

El diseño sale de las media queries: el móvil por debajo de 900 px y el PC desde 900 px. En el PC, la interfaz guarda las 3 posiciones de los tiradores en la clave `<carpeta>-columnas` de `localStorage`.

## La WASM

`npm run wasm` compila `nucleo` a WASM, y `wasm-bindgen` deja el módulo y sus tipos de TypeScript en `interfaz/src/nucleo/`, que no entra en git. Los tipos salen de `tsify`. `npm run build` y `npm run dev` corren antes `npm run wasm`, así que un cambio de un tipo en Rust rompe la build de la interfaz.

## Los tests

`cargo test` corre los tests de `nucleo` y del servidor. Cada test del servidor crea una raíz temporal de exámenes, arranca el servidor y le habla por HTTP.

`npm run comprobar`, después de `npm run build`, corre en Playwright cada pantalla en el ancho del móvil y en el del PC, contra el servidor y una copia de `ln/`. Las capturas quedan en `interfaz/capturas/`, que no entra en git.

En desarrollo, `npm run dev` sirve la interfaz con Vite y pasa el API al servidor de `127.0.0.1:8601`.
