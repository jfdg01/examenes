# La migración del test a Rust y Svelte 5

Este doc es el registro de la migración: el orden del trabajo, lo que cambia en la app y las preguntas abiertas. Las decisiones del stack y su razón están en [el ADR 0001](adr/0001-el-test-en-rust-y-svelte.md).

## El orden

1. `feat/varios-examenes` entra en `main`. El historial de LN pasa de `ln/` en `main` a la carpeta `ln/` del examen, y los PDFs sueltos y el código viejo salen de `ln/`.
2. La migración vive en la rama `feat/rust-svelte`. La versión de Python sigue en uso hasta que la versión de Rust hace lo mismo.
3. Un commit final borra `test/` entero.

Antes del cambio de URL, envía al servidor las respuestas pendientes del móvil y del PC. El origen nuevo tiene otro `localStorage`, y las respuestas sin enviar se quedan en el origen viejo.

## Cada función se revisa antes de migrar

Una función que nadie usa no pasa a Rust, y su código se va. Una función que no se ve por un fallo se arregla, y no se borra. Cada función se comprueba en un navegador, en el móvil y en el PC, antes de decidir.

## Lo que cambia en la app

- **El diseño sale del CSS.** Las media queries eligen el diseño del móvil o del PC, y el diseño cambia con el ancho de la ventana. El contenido de cada diseño queda igual:
  - En el móvil: el plegado de las opciones, y la página del PDF a pantalla completa.
  - En el PC: la página al lado de la pregunta, el concepto y el dominio.
- **Los tiradores de las columnas se quedan en el PC.** Mueven los bordes y el separador de las 2 columnas.
- **Un banco vacío no muestra la sesión ni el simulacro.** El inicio dice "El banco no tiene preguntas".
- **El API envía solo lo que la pantalla muestra.** Un valor intermedio de un cálculo se queda en Rust.
- **El servidor no tiene los flags `--banco` ni `--historial`.** Cada test de Rust crea una carpeta de examen temporal con sus archivos. `ln/preguntas-ejemplo.json` pasa a la carpeta de los tests de Rust.

## Las preguntas abiertas

- **El botón de los tiradores.** Los tiradores solo se activan con el botón del engranaje, arriba de la pregunta en el PC, y el dueño no reconoce ese botón. Falta decidir cómo se ven los tiradores.
- **El bloque de estudio.** Un bloque de estudio termina con una pausa de 3 horas sin respuestas. Una pregunta de la caja 2 o de la caja 3 no sale otra vez en el bloque de su última respuesta. Falta decidir si el bloque se queda, y qué dice la pantalla cuando no hay preguntas pendientes.

## Deletion

Borrar este doc cuando la migracion este completa y no sea necesario.
