# El grilling de la lógica en el navegador

Esta nota prepara un `grill-with-docs`. El grilling decide dónde corre la lógica de la app, y cómo guarda el historial una pregunta en blanco. Las respuestas van a un ADR nuevo, y esta nota se borra después.

## La propuesta

`nucleo` compila a WebAssembly (WASM) y corre también en el navegador. La interfaz llama a `nucleo` para corregir una respuesta, cerrar el simulacro, calcular la nota y unir los historiales, también sin red. El servidor guarda los archivos de cada examen y los sirve.

## Lo que la pide

- `las-respuestas-sin-red-en-rust` (#24): el móvil corrige y guarda una respuesta sin el servidor.
- `la-ronda-a-medias-en-rust` (#23): la ronda y su límite sobreviven a una recarga.
- `el-simulacro-en-rust` (#22) deja hoy lógica en la interfaz:
  - `App.svelte` juzga cada respuesta con `eleccion?.opcion === q?.correcta`.
  - `App.svelte` guarda los `resultados` del simulacro en memoria, y los manda a `POST api/cierre`. El servidor confía en esa lista y en `Cierre.preguntas`.
  - El límite del simulacro vive solo en `App.svelte`, y ningún test de `cargo test` lo cubre.
- El simulacro tiene 2 verdades: los `resultados` de la interfaz y `respuestas.jsonl`.

## Lo que choca

- El ADR 0001 dice "la lógica vive en Rust, en el servidor".
- El ADR 0001 descarta la WASM con esta razón: "Sin el servidor la página tampoco carga". #24 rompe esa razón: la página carga, y la red se va después, en mitad de una ronda.
- El ADR 0001 dice "el formato de los datos de un examen no cambia". El punto del historial cambia `respuestas.jsonl`.

## El historial y la pregunta en blanco

`respuestas.jsonl` guarda una pregunta en blanco como `{"acierto": false, "seguro": false}`, igual que un fallo con "dudo". Así, el servidor no puede calcular la nota desde el historial, y `el-modelo-de-la-nota` (#36) no puede medir si dejar en blanco sale a cuenta.

En LN, 55 de las 312 respuestas tienen `acierto: false, seguro: false`. Ninguna pista dice cuáles fueron en blanco, así que una migración no las puede separar.
