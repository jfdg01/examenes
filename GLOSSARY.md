# Los exámenes

Una app para practicar exámenes, con un banco de preguntas de test y de desarrollo por examen.

## El examen y su banco

**Examen**: Una asignatura con su formato: el número de preguntas de test y de desarrollo, de opciones y de minutos, la penalización de un fallo y el peso de cada parte en la nota. _Evita_: test, asignatura

**Banco**: Las preguntas de un examen, de test y de desarrollo.

**Pregunta**: Una pregunta de test o una pregunta de desarrollo. El estado de una pregunta vale para las dos.

**Pregunta de test**: Un enunciado con una opción correcta y sus distractores.

**Pregunta de desarrollo**: Un enunciado que pide escribir de memoria una definición o las características de algo, como "Describe el handshake de SSH". Junta de 4 a 8 conceptos de un apartado, sus puntos. _Evita_: tema, concepto

**Respuesta modelo**: La respuesta en prosa de una pregunta de desarrollo, la que memorizas.

**Punto**: Un concepto que la respuesta a una pregunta de desarrollo tiene que cubrir. _Evita_: criterio

**Tema**: Un PDF del temario, como lo numera la clase. Un tema trae muchos conceptos. _Evita_: lección, unidad

**Apartado**: Un trozo de un tema, con su nombre corto. Un tema tiene uno o varios apartados, y un apartado puede salir en varios temas. _Evita_: tema, sección

**Concepto**: Una idea del temario, escrita en una frase que se entiende sola, con su página. Del concepto salen sus preguntas de test, cada una desde un ángulo, y es un punto de una pregunta de desarrollo como mucho. Una pregunta de test da pistas de las otras de su concepto, y por eso una sesión de test lleva una sola. _Evita_: tema (un tema trae muchos conceptos)

**Hermana**: Otra pregunta de test del mismo concepto.

**Historial**: Todas las respuestas de un examen, en orden de hora. Una respuesta de test es un acierto, un fallo o una pregunta en blanco. Una respuesta de desarrollo lleva su corrección, o está sin corregir.

## La respuesta

**Acierto con "seguro"**: Una respuesta correcta, marcada como segura antes de ver la solución. Tras él, la probabilidad de acierto baja más despacio que tras un acierto con "dudo".

**Corrección**: La nota de una respuesta de desarrollo: no la sabías, a medias o la sabías. La pone una persona o un agente, y la más nueva gana.

**Sin corregir**: Una respuesta de desarrollo sin corrección. No mueve la probabilidad de acierto hasta que alguien la corrige. _Evita_: pendiente, borrador

**En blanco**: Una pregunta que dejas sin contestar a propósito. Entra en el historial, y cuenta como un fallo. Una pregunta sin contestar al pasar el límite del simulacro no entra en el historial.

## El estado de una pregunta

**Sin ver**: Una pregunta sin ninguna respuesta registrada. _Evita_: nuevo

**Probabilidad de acierto**: Lo que el planificador estima que aciertas una pregunta ahora. Baja con el tiempo desde cada respuesta, más despacio cuanto mejor la sabes. _Evita_: retención (es la probabilidad pedida), paso, nivel

**Retención**: La probabilidad de acierto con la que vence una pregunta. Una retención más alta hace vencer antes las preguntas, y pide más repasos.

**Vencida**: Una pregunta que toca repasar: una fallada, o una pregunta vista con la probabilidad de acierto por debajo de la retención. _Evita_: hueco, intervalo

**Fallada**: Una pregunta cuya última respuesta es un fallo, una pregunta en blanco o una pregunta de desarrollo que no sabías. Una fallada está siempre vencida. _Evita_: caja, pendiente

**Sabes**: Una pregunta vista y no vencida: su probabilidad de acierto está por encima de la retención. _Evita_: dominado, sabido

## La práctica

**Ronda**: Una serie de preguntas de principio a fin: una sesión de test, una sesión de desarrollo o un simulacro.

**Sesión de test**: Una ronda de 12 preguntas de test, como mucho una por concepto de la selección: primero las falladas, y después las vencidas, de la menor probabilidad de acierto a la mayor. _Evita_: repaso, bloque

**Sesión de desarrollo**: Una ronda de 4 preguntas de desarrollo de la selección: primero las falladas, y después las vencidas.

**Simulacro**: Una ronda con el formato del examen: sus preguntas de test y de desarrollo al azar de la selección, y su tiempo.

**Selección**: Los apartados de cada tema que marcas en el inicio. Las rondas y la tarjeta de la nota usan solo sus preguntas. Sin nada marcado antes, la selección es el banco entero. _Evita_: filtro
