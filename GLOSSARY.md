# Los exámenes

Una app para practicar exámenes tipo test, con un banco de preguntas por examen.

## El examen y su banco

**Examen**: Una asignatura con su formato: el número de preguntas, de opciones y de minutos, y la penalización de un fallo. _Evita_: test, asignatura

**Banco**: Las preguntas de un examen.

**Tema**: Un PDF del temario, como lo numera la clase. Un tema trae muchos conceptos. _Evita_: lección, unidad

**Apartado**: Un trozo de un tema, con su nombre corto. Un tema tiene uno o varios apartados, y un apartado puede salir en varios temas. _Evita_: tema, sección

**Concepto**: La idea que una pregunta pone a prueba. Varias preguntas del banco comparten un concepto, cada una desde un ángulo. Una pregunta da pistas de las otras de su concepto, y por eso una sesión lleva una sola. _Evita_: tema (un tema trae muchos conceptos)

**Hermana**: Otra pregunta del mismo concepto.

**Historial**: Todas las respuestas de un examen, en orden de hora. Cada respuesta es un acierto, un fallo o una pregunta en blanco.

## La respuesta

**Acierto con "seguro"**: Una respuesta correcta, marcada como segura antes de ver la solución. Tras él, la probabilidad de acierto baja más despacio que tras un acierto con "dudo".

**En blanco**: Una pregunta que dejas sin contestar a propósito. Entra en el historial, y cuenta como un fallo. Una pregunta sin contestar al pasar el límite del simulacro no entra en el historial.

## El estado de una pregunta

**Sin ver**: Una pregunta sin ninguna respuesta registrada. _Evita_: nuevo

**Probabilidad de acierto**: Lo que el planificador estima que aciertas una pregunta ahora. Baja con el tiempo desde cada respuesta, más despacio cuanto mejor la sabes. _Evita_: retención (es la probabilidad pedida), paso, nivel

**Retención**: La probabilidad de acierto con la que vence una pregunta. Una retención más alta hace vencer antes las preguntas, y pide más repasos.

**Vencida**: Una pregunta que toca repasar: una fallada, o una pregunta vista con la probabilidad de acierto por debajo de la retención. _Evita_: hueco, intervalo

**Fallada**: Una pregunta cuya última respuesta es un fallo o una pregunta en blanco. Una fallada está siempre vencida. _Evita_: caja, pendiente

**Sabes**: Una pregunta vista y no vencida: su probabilidad de acierto está por encima de la retención. _Evita_: dominado, sabido

## La práctica

**Ronda**: Una serie de preguntas de principio a fin: una sesión o un simulacro.

**Sesión**: Una ronda de 12 preguntas, como mucho una por concepto de la selección: primero las falladas, y después las vencidas, de la menor probabilidad de acierto a la mayor. _Evita_: repaso, bloque

**Simulacro**: Una ronda con el formato del examen: sus preguntas al azar de la selección, y su tiempo.

**Selección**: Los apartados de cada tema que marcas en el inicio. Las rondas y la tarjeta de la nota usan solo sus preguntas. Sin nada marcado antes, la selección es el banco entero. _Evita_: filtro
