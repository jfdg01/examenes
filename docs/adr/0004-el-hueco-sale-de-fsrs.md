# El hueco sale de FSRS, con el tope de la víspera del examen

El estudiante practica para un examen con fecha, a menudo con pocos días por delante. Un repaso rinde más con un hueco proporcional a la distancia al examen (Cepeda et al., 2008). Las preguntas de un concepto no son igual de difíciles para el estudiante ([el ADR 0003](0003-el-estado-es-de-la-pregunta.md)).

## Las decisiones

- **El hueco sale de FSRS, con el crate `fsrs` 7.** FSRS aprende del historial la dificultad de cada pregunta y su recuerdo, y fija el hueco para un recuerdo del 90 %. Una pregunta difícil vuelve más a menudo que una fácil del mismo concepto. `fsrs` es el crate oficial, el mismo que usa Anki, y trae el planificador y el optimizador. Compila a WASM con `getrandom` y su feature `wasm_js`.
- **Un acierto con "dudo" es Again, y no Hard.** Un "dudo" en un tipo test es muchas veces una adivinanza entre 2 opciones, y acertar por suerte no es recordar. Anki pide Again para una respuesta adivinada. Solo un acierto con "seguro" es Good.
- **Los pasos de aprendizaje son de 1 y 10 minutos, como en Anki, y un solo esquema sirve para las preguntas nuevas y las falladas.** Una fallada vuelve pronto, y una pregunta nueva no espera días tras su primer acierto.
- **Los parámetros son comunes a todos los exámenes.** El estudiante es el mismo, con el mismo tipo de pregunta, y los historiales juntos llegan antes al mínimo del optimizador. El optimizador es una función pura de `nucleo`, y el servidor la llama porque es el único que lee todos los historiales.
- **Ningún hueco pasa del principio de la víspera del examen.** FSRS no sabe cuándo es el examen, y tras unos pocos aciertos da huecos de semanas. El tope da a cada pregunta un repaso el día antes. Una respuesta posterior a ese momento sigue solo a FSRS, para que el tope no la devuelva sin fin.
- **"Sabes" es un recuerdo del 90 % o más el día del examen, fuera de los pasos de aprendizaje.** La barra mide lo que el estudiante sabrá en el examen, y no lo que sabe hoy.

