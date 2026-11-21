# La migración de la app a Rust y Svelte 5

Este doc es el registro de la migración: el orden del trabajo y lo que cambia en la app. Las decisiones del stack y su razón están en [el ADR 0001](adr/0001-la-app-en-rust-y-svelte.md).

## El orden

1. `feat/varios-examenes` entra en `main`. El historial de LN pasa de `ln/` en `main` a la carpeta `ln/` del examen, y los PDFs sueltos y el código viejo salen de `ln/`.
2. La migración vive en la rama `feat/rust-svelte`. La versión de Python sigue en uso hasta que la versión de Rust hace lo mismo.
3. Un commit final borra `test/` entero.

El código nuevo vive en `app/`. La carpeta no se llama `test/`, porque ese nombre se confunde con los tests del código.

Antes del cambio de URL, envía al servidor las respuestas pendientes del móvil y del PC. El origen nuevo tiene otro `localStorage`, y las respuestas sin enviar se quedan en el origen viejo.

## Cada función se revisa antes de migrar

Una función que nadie usa no pasa a Rust, y su código se va. Una función que no se ve por un fallo se arregla, y no se borra. Cada función se comprueba en un navegador, en el móvil y en el PC, antes de decidir.

## Lo que cambia en la app

- **El diseño sale del CSS.** Las media queries eligen el diseño del móvil o del PC, y el diseño cambia con el ancho de la ventana. El contenido de cada diseño queda igual:
  - En el móvil: el plegado de las opciones, y la página del PDF a pantalla completa.
  - En el PC: la página al lado de la pregunta, el concepto y el dominio.
- **Los tiradores de las columnas se quedan en el PC.** Mueven los bordes y el separador de las 2 columnas. Cada tirador muestra su asa cuando el ratón pasa por encima, y se arrastra siempre. El PC no tiene un modo de edición ni el botón del engranaje.
- **La sesión no tiene un bloque de estudio.** Una pregunta de la caja 2 o de la caja 3 puede salir en la sesión siguiente, sin una pausa.
- **Un banco vacío no muestra la sesión ni el simulacro.** El inicio dice "El banco no tiene preguntas".
- **El API envía solo lo que la pantalla muestra.** Un valor intermedio de un cálculo se queda en Rust.
- **El servidor no tiene los flags `--banco` ni `--historial`.** Cada test de Rust crea una carpeta de examen temporal con sus archivos. `ln/preguntas-ejemplo.json` pasa a la carpeta de los tests de Rust.

## El final de este doc

Este doc se borra cuando la migración termina.
