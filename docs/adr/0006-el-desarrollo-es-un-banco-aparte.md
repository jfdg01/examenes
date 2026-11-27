# El desarrollo es un banco aparte, con su historial y su estado

Una pregunta de desarrollo entrena el recuerdo de una idea grande, y una pregunta de test el reconocimiento de una pequeña. Una nota sobre "el handshake de SSH" dice poco de cada pregunta de test de SSH.

## Las decisiones

- **Cada pregunta de desarrollo tiene su estado de FSRS**, con la misma retención de `examen.json`. Una corrección no mueve el estado de ninguna pregunta de test, ni al revés.
- **El banco va en `desarrollo.json`, y el historial en `desarrollo.jsonl`.** Un formato por archivo, y ningún lector del test filtra respuestas de desarrollo.
- **La nota es la de FSRS: Again, Hard o Good.** Sin Easy, como en el [ADR 0004](0004-el-planificador-es-fsrs.md).
- **La corrección la pone una persona o un agente, fuera del motor de la app.** La app no llama a un LLM: pide una clave, red y un coste por respuesta, y un error del modelo enseña algo falso. Un agente corrige con `banco pendientes` y `banco corregir`.
- **`desarrollo.jsonl` solo añade líneas.** Una respuesta es `{"id", "ts", "texto"}`, y una corrección es `{"id", "ts", "nota", "corregida"}`, con el `ts` de su respuesta. FSRS cuenta una respuesta corregida a la hora en que respondiste, y la corrección más nueva gana. Una respuesta sin corregir no mueve el estado.

## Las alternativas

- **El concepto como pregunta de desarrollo.** Una idea pequeña no entrena una respuesta larga.
- **Un estado compartido con el test.** Una nota grande movería preguntas pequeñas que no se probaron.
- **Un LLM que corrige dentro de la app.** Ver la tercera decisión.
- **La corrección reescribe la línea de su respuesta.** El servidor y el móvil unen el historial por `id` y `ts`. Con 2 versiones de la misma línea, la unión necesita una regla de quién gana, y una corrección vieja puede borrar una nueva.
- **Solo una respuesta modelo, o solo una lista de puntos.** La prosa se memoriza, y la lista se comprueba. La solución enseña las dos.
