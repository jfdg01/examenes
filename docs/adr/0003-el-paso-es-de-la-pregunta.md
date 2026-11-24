# El paso es de la pregunta, y el concepto solo agrupa

El banco escribe cada idea como un concepto con 3 preguntas, desde 3 ángulos. Las 3 preguntas de un concepto no son igual de difíciles, y la difícil suele probar una parte de la idea que la fácil no toca.

## Las decisiones

- **Cada pregunta tiene su paso, su hueco y su vencimiento.** Una respuesta mueve solo el paso de su pregunta. Con un paso por concepto, un fallo en la difícil pone el paso a 0, un acierto en la fácil lo sube, y la rotación vuelve a la difícil: el concepto no avanza nunca, y la difícil no se aprende. Con un paso por pregunta, la difícil vuelve hasta que la aciertas, y la fácil sigue su camino.
- **El concepto agrupa.** Una sesión lleva como mucho una pregunta por concepto, y un simulacro también mientras haya conceptos. Una hermana da pistas de la respuesta, y un acierto con pista no mide lo que sabes. Es la misma regla que Anki aplica a las tarjetas hermanas.
- **"Sabes", la barra, la tarjeta y el inicio cuentan preguntas.** El examen pregunta preguntas, y una sola unidad deja que las cifras sumen.
- **El hueco es de 1 día con el paso 1, y ×2,5 con cada paso más.** Un paso por pregunta multiplica los repasos casi por 3. Un hueco de minutos gasta en repasos el tiempo que piden las preguntas sin ver: con el banco entero por delante y pocos días, una vuelta a los 10 minutos sale bien un 93 % de las veces y casi no enseña.
- **Una pregunta sin ver acertada con "seguro" salta al paso 3**, que vence en unos 6 días. Una pregunta nueva cuesta así una respuesta, y un fallo cuesta 3: el fallo, el acierto y la vuelta del día siguiente.
- **El historial no migra.** Cada respuesta de `respuestas.jsonl` guarda el `id` de su pregunta, y el estado de cada pregunta sale de ese historial.

## Las alternativas

- **Igualar la dificultad de las preguntas de un concepto.** Pide reescribir los bancos, y la dificultad depende del estudiante: 2 preguntas nunca son igual de difíciles para todos.
- **El paso en el concepto, y la pregunta fallada vuelve hasta acertarla.** Corta el bucle con pocas líneas, pero una pregunta fácil sigue subiendo el paso de una difícil.
- **El concepto en "sabes" cuando todas sus preguntas lo están.** Con 2 de 3 preguntas en "sabes", el concepto cuenta 0, y la barra avanza a saltos.
- **FSRS, el planificador de Anki que aprende la dificultad de cada pregunta.** Es el paso siguiente, en la issue #50. Cambia qué significa "sabes" y cómo predice la nota, y merece su propio diseño.
