# El estado es de la pregunta, y el concepto solo agrupa

El banco escribe cada idea como un concepto con 3 preguntas, desde 3 ángulos. Las 3 preguntas de un concepto no son igual de difíciles, y la difícil suele probar una parte de la idea que la fácil no toca.

## Las decisiones

- **Cada pregunta tiene su estado, su hueco y su vencimiento.** Una respuesta mueve solo el estado de su pregunta. Con un estado por concepto, un fallo en la difícil lo baja, un acierto en la fácil lo sube, y la rotación vuelve a la difícil: el concepto no avanza nunca, y la difícil no se aprende. Con un estado por pregunta, la difícil vuelve hasta que la aciertas, y la fácil sigue su camino.
- **El concepto agrupa.** Una sesión lleva como mucho una pregunta por concepto, y un simulacro también mientras haya conceptos. Una hermana da pistas de la respuesta, y un acierto con pista no mide lo que sabes. Es la misma regla que Anki aplica a las tarjetas hermanas.
- **"Sabes", la barra, la tarjeta y el inicio cuentan preguntas.** El examen pregunta preguntas, y una sola unidad deja que las cifras sumen.
- **El historial no migra.** Cada respuesta de `respuestas.jsonl` guarda el `id` de su pregunta, y el estado de cada pregunta sale de ese historial.

Cómo sale el hueco del estado está en [el ADR 0004](0004-el-hueco-sale-de-fsrs.md).

## Las alternativas

- **Igualar la dificultad de las preguntas de un concepto.** Pide reescribir los bancos, y la dificultad depende del estudiante: 2 preguntas nunca son igual de difíciles para todos.
- **El estado en el concepto, y la pregunta fallada vuelve hasta acertarla.** Corta el bucle con pocas líneas, pero un acierto en una pregunta fácil sigue subiendo el estado de una difícil.
- **El concepto en "sabes" cuando todas sus preguntas lo están.** Con 2 de 3 preguntas en "sabes", el concepto cuenta 0, y la barra avanza a saltos.
