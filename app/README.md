# La app de práctica

Un servidor de Rust y una interfaz de Svelte 5. Las decisiones del stack están en [el ADR 0001](../docs/adr/0001-la-app-en-rust-y-svelte.md), y dónde corre la lógica está en [el ADR 0002](../docs/adr/0002-el-nucleo-en-wasm.md). Los diagramas de las clases, la ronda y una sesión están en PlantUML, en [el diagrama](../docs/el-diagrama.puml).

- `nucleo/`: toda la lógica, sin leer archivos ni la red. Corre en el navegador como WASM, y el servidor usa sus tipos para validar.
- `servidor/`: el servidor de `axum`. Guarda y sirve los archivos de cada examen, con gzip.
- `banco/`: las herramientas del banco de un examen, para su dueño. `banco comprobar <carpeta>` comprueba `preguntas.json` con las reglas de [`BANCO.md`](../test/BANCO.md), `banco paginas <carpeta>` saca las imágenes de `paginas/` de los PDFs de `pdfs/`, y `banco revision <carpeta> <informe>...` aplica los informes de una revisión a `preguntas.json`. `comprobar` y `paginas` necesitan `pdfinfo` y `pdftoppm`, de poppler.
- `interfaz/`: la interfaz de Svelte 5, con Vite, TypeScript y Tailwind 4. `main.ts` monta `Lista.svelte` en `/` y `App.svelte` en `/<carpeta>/`. `App.svelte` guarda el estado de la ronda y llama a `nucleo` para cada cálculo. Los componentes `Inicio`, `Pregunta`, `Hoja`, `Tiradores`, `PaginaCompleta` y `Final` reciben sus datos por props y avisan con callbacks.

## Arrancar

La build de la interfaz necesita el target `wasm32-unknown-unknown` y `wasm-bindgen-cli`, con la versión de `wasm-bindgen` de `nucleo/Cargo.toml`.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
```

```bash
cd app/interfaz && npm install && npm run build
cd .. && cargo run -p servidor -- .. 127.0.0.1:8602
tailscale serve --bg --https=8601 http://127.0.0.1:8602
```

El servidor recibe la raíz de los exámenes y su dirección. Sirve cada carpeta de la raíz con `examen.json` en `/<carpeta>/`, y lee la carpeta en cada petición. Un examen nuevo aparece al crear su carpeta, sin reiniciar el servidor. Un campo de `examen.json`, `preguntas.json`, `respuestas.jsonl` o `ronda.json` que su tipo de `nucleo` no conoce da un error que lo nombra. Cada hora de los datos va en ISO 8601 con `Z`, como `2026-11-04T12:41:22Z`: `ts` de `respuestas.jsonl` y `guardada` de `ronda.json`.

`/` muestra la lista de los exámenes, y `GET /api/examenes` da cada examen con su `carpeta` y su `titulo`, en orden alfabético por título, sin tener en cuenta las tildes ni las mayúsculas. El inicio de cada examen lleva el enlace «← Exámenes» de vuelta a la lista. Cada ruta de abajo cuelga de `/<carpeta>/`.

- `GET api/inicio` da el examen, el banco y el historial.
- `POST api/respuestas` añade al historial cada respuesta que no está. Una línea de `respuestas.jsonl` lleva `id`, `ts` (la hora), `resultado` (`Acierto`, `Fallo` o `Blanco`) y `seguro`. La interfaz guarda cada respuesta en la clave `<carpeta>-pendientes` de `localStorage` hasta que llega al servidor. Al cargar la página y al volver la red, envía las que esperan.
- `GET api/ronda` da `ronda.json`, y `PUT api/ronda` escribe el guardado más nuevo entre el que llega y el de `ronda.json`: `{"guardada", "ronda"}`, con la hora del guardado y la ronda a medias o `null`. La interfaz guarda la misma ronda en la clave `<carpeta>-ronda` de `localStorage`, y al cargar envía el guardado más nuevo de los dos.
- `GET api/notas` da `notas.json`, un objeto de `id` a texto. `POST api/notas` recibe `{"id", "texto"}` y guarda la nota de esa pregunta, o la borra si el texto está en blanco. La interfaz envía la nota medio segundo después de la última tecla en su pregunta. El botón ✎ muestra u oculta la nota, y guarda la elección en la clave `<carpeta>-notas-visibles` de `localStorage`. Sin la clave, la nota se ve en el PC y no en el móvil.
- `GET paginas/<archivo>` da la imagen de una página del PDF, y `GET pdfs/<archivo>` da un PDF. Los 2 leen la carpeta `paginas/` o `pdfs/` del examen, y dan 404 a un nombre con `/` o con un punto delante.

El diseño sale de las clases de Tailwind: el móvil por defecto, y el PC desde 900 px con el breakpoint `pc:` de `src/app.css`. En el PC, la interfaz guarda las 3 posiciones de los tiradores en la clave `<carpeta>-columnas` de `localStorage`.

## La WASM

`npm run wasm` compila `nucleo` a WASM, y `wasm-bindgen` deja el módulo y sus tipos de TypeScript en `interfaz/src/nucleo/`, que no entra en git. Los tipos salen de `tsify`. `npm run build` y `npm run dev` corren antes `npm run wasm`, así que un cambio de un tipo en Rust rompe la build de la interfaz.

## Los tests

`cargo test` corre los tests de `nucleo`, del servidor y de `banco`. Los tests de `banco` crean una carpeta temporal de examen con un PDF mínimo. Cada test del servidor crea una raíz temporal de exámenes, arranca el servidor y le habla por HTTP.

`npm run comprobar`, después de `npm run build`, corre en Playwright cada pantalla en el ancho del móvil y en el del PC, contra el servidor y una copia de `ln/`. Las capturas quedan en `interfaz/capturas/`, que no entra en git. La comprobación empieza por la lista, con la raíz de las 2 copias y con una raíz vacía. Las aserciones buscan las clases `examen`, `volver`, `tirador`, `hoja`, `chip`, `opcion`, `icono`, `ver-pagina`, `pagina`, `vacia`, `nota`, `ver-nota` y `copiar`, que no llevan estilo.

En desarrollo, `npm run dev` sirve la interfaz con Vite y pasa la lista y el API de cada examen al servidor de `127.0.0.1:8602`.
