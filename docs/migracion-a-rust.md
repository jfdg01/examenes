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

- **El diseño sale del CSS.** Las media queries eligen el diseño del móvil o del PC, y el diseño cambia con el ancho de la ventana. Los 2 diseños muestran la misma información, y solo cambia su colocación:
  - En el móvil: el plegado de las opciones, y la página del PDF a pantalla completa.
  - En el PC: la página al lado de la pregunta.
- **Los tiradores de las columnas se quedan en el PC.** Mueven los bordes y el separador de las 2 columnas. Cada tirador muestra su asa cuando el ratón pasa por encima, y se arrastra siempre. El PC no tiene un modo de edición ni el botón del engranaje.
- **La sesión elige conceptos por su fecha de repaso.** Las cajas de las preguntas, sus pesos y el bloque de estudio desaparecen. El detalle está en [el modelo](el-modelo.md).
- **Un banco vacío no muestra la sesión ni el simulacro.** El inicio dice "El banco no tiene preguntas".
- **El API envía solo lo que la pantalla muestra.** Un valor intermedio de un cálculo se queda en Rust.
- **El servidor no tiene los flags `--banco` ni `--historial`.** Cada test de Rust crea una carpeta de examen temporal con sus archivos. `ln/preguntas-ejemplo.json` pasa a la carpeta de los tests de Rust.

## El flujo de trabajo

1. El grilling termina con la revisión de la lógica de [el modelo](el-modelo.md): el paso y la sesión, que la app de JavaScript prueba antes, el estado del concepto y la ronda.
2. `/to-spec` convierte las decisiones en un spec, y `/to-tickets` lo corta en tickets con sus dependencias.
3. Cada ticket se construye con un `/implement`, con `/clear` entre ticket y ticket.

Al final del grilling, cada doc que no hace falta se borra, y cada línea que no hace falta sale de los docs. Al final de la implementación, un doc que el código ya explica se resume a lo que el código no dice.

La rama `feat/bkt` y su modelo BKT quedan fuera de la migración, congelados hasta que el dueño los nombre. La migración pasa a Rust el modelo de los intervalos.

## Las preguntas abiertas

- **El estado del concepto en la tarjeta de la nota.** Hoy "sabes" sale de la última respuesta, y no del paso.
- **La ronda.** Falta confirmar su máquina en [el modelo](el-modelo.md).
- **Las máquinas de estados explícitas.** El spec define cada estado y cada transición, y Rust tiene un `enum` por máquina.

## El final de este doc

Este doc se borra cuando la migración termina.
