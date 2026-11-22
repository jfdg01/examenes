# `nucleo` corre en WASM, en el navegador y en el servidor

La app corrige, guarda y calcula sin red. Una ronda empieza con la página cargada, y la red se puede ir en mitad de la ronda.

## Las decisiones

- **Todo `nucleo` compila a WebAssembly (WASM).** La interfaz llama a `nucleo` para corregir una respuesta, elegir la sesión y el simulacro, contar el límite, cerrar el simulacro, calcular la nota y la tarjeta, unir 2 historiales y elegir la ronda que gana. La interfaz no decide nada del examen: cada regla, como el paso tras una respuesta o su aviso, vive solo en `nucleo`. La interfaz decide en TypeScript solo cómo se ve, como las marcas ✓ y ✗, el reloj en m:ss y el ancho de una barra, porque un cambio visual no debe pasar por Rust.
- **El navegador guarda en `localStorage` las respuestas que no llegaron al servidor.** Una respuesta sin red espera ahí, y sale al servidor cuando vuelve la red. Después el navegador lee otra vez el historial, y `nucleo` lo une con las respuestas que esperan. El banco no va a `localStorage`: sin service worker, una recarga sin red no carga la página, y la ronda ya tiene el banco en memoria.
- **El servidor guarda y sirve archivos.** Sus rutas son `GET api/inicio`, `POST api/respuestas`, el `GET` y el `PUT` de `ronda.json`, el `GET` y el `POST` de `notas.json`, y el `GET` de las imágenes de `paginas/` y de los PDFs de `pdfs/`. El servidor usa `nucleo` para validar lo que recibe, y comprime con gzip.
- **La build usa `wasm-bindgen-cli`.** `cargo build --target wasm32-unknown-unknown` y `wasm-bindgen --target web` corren en el `prebuild` de `npm run build`. La versión de `wasm-bindgen-cli` es igual a la del crate `wasm-bindgen` de `Cargo.lock`.
- **Los tipos de TypeScript salen de `tsify`.** `tsify` escribe los tipos de los datos y de las firmas en el `.d.ts` de `wasm-bindgen`. Un cambio de una firma en Rust rompe la build de la interfaz, y no la app en el móvil.
- **`respuestas.jsonl` guarda el resultado de cada respuesta**: `{"id", "ts", "resultado", "seguro"}`, con `resultado` igual a `Acierto`, `Fallo` o `Blanco`. Con un campo `acierto`, una pregunta en blanco y un fallo con "dudo" se guardan igual, y la nota no sale del historial. Las respuestas anteriores al campo `resultado` pasan a `Acierto` o a `Fallo`.

## Las alternativas

- **El servidor manda en la ronda.** La interfaz envía `{id, opcion}` y el servidor corrige. Sin red, la app no corrige.
- **La lógica en el servidor, y una regla escrita sobre lo que calcula la interfaz.** La interfaz corrige y cuenta el límite en TypeScript, sin `cargo test`. Sin red, un simulacro no da su nota y una sesión nueva no empieza.
- **Solo una parte de `nucleo` en WASM.** Una función que no cruza vive solo en el servidor, y sin red no corre.
- **`wasm-pack`.** No tiene mantenimiento desde 2025.
- **`ts-rs`, con un archivo de TypeScript que pone los tipos a las funciones de la WASM.** Ese archivo puede quedar viejo sin que la build lo note.
- **Un campo `blanco: true` al lado de `acierto`.** Permite una combinación imposible, como `acierto: true, blanco: true`.
