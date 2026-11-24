# Los exámenes

Una app para practicar exámenes tipo test, con un banco de preguntas por examen.

## El examen y su banco

**Examen**: Una asignatura con su formato y su fecha: el número de preguntas, de opciones y de minutos, y la penalización de un fallo. _Evita_: test, asignatura

**Banco**: Las preguntas de un examen.

**Tema**: Un PDF del temario, como lo numera la clase. Un tema trae muchos conceptos. _Evita_: lección, unidad

**Apartado**: Un trozo de un tema, con su nombre corto. Un tema tiene uno o varios apartados, y un apartado puede salir en varios temas. _Evita_: tema, sección

**Concepto**: La idea que una pregunta pone a prueba. Varias preguntas del banco comparten un concepto, cada una desde un ángulo. Una pregunta da pistas de las otras de su concepto, y por eso una sesión lleva una sola. _Evita_: tema (un tema trae muchos conceptos)

**Hermana**: Otra pregunta del mismo concepto.

**Historial**: Todas las respuestas de un examen, en orden de hora. Cada respuesta es un acierto, un fallo o una pregunta en blanco.

## La respuesta

**Acierto con "seguro"**: Una respuesta correcta, marcada como segura antes de ver la solución. Es la única respuesta que cuenta como recordada.

**En blanco**: Una pregunta que dejas sin contestar a propósito. Entra en el historial, y cuenta como un fallo. Una pregunta sin contestar al pasar el límite del simulacro no entra en el historial.

## El estado de una pregunta

**Sin ver**: Una pregunta sin ninguna respuesta registrada. _Evita_: nuevo

**Recuerdo**: La probabilidad de acertar una pregunta en un momento dado. Baja con el tiempo desde la última respuesta, y baja más despacio en una pregunta acertada muchas veces. _Evita_: nivel, retención

**Hueco**: El tiempo entre una respuesta y el vencimiento de la pregunta. Fuera de los pasos de aprendizaje, es lo que tarda su recuerdo en bajar al 90 %. Nunca pasa del principio de la víspera del examen. _Evita_: intervalo, pausa

**Pasos de aprendizaje**: Los repasos cortos de una pregunta nueva o fallada antes de que el hueco crezca con su recuerdo. Una fallada vuelve a 1 minuto, y un acierto con "seguro" la lleva a 10 minutos. Otro acierto con "seguro" a los 10 minutos la saca de los pasos.

**Vencida**: Una pregunta cuyo hueco ya pasó, y que toca repasar.

**Fallada**: Una pregunta vista cuya última respuesta es un fallo, un acierto con "dudo" o una respuesta en blanco. Una fallada vence 1 minuto después de su respuesta. _Evita_: caja, pendiente

**Sabes**: Una pregunta con un recuerdo del 90 % o más el día del examen, o hoy si el examen no tiene fecha o ya pasó. _Evita_: dominado, sabido

## La práctica

**Ronda**: Una serie de preguntas de principio a fin: una sesión o un simulacro.

**Sesión**: Una ronda de 12 preguntas, como mucho una por concepto de la selección: primero las falladas, después las vencidas de menor a mayor recuerdo, y después las demás. _Evita_: repaso, bloque

**Simulacro**: Una ronda con el formato del examen: sus preguntas al azar de la selección, y su tiempo.

**Selección**: Los apartados de cada tema que marcas en el inicio. Las rondas y la tarjeta de la nota usan solo sus preguntas. Sin nada marcado antes, la selección es el banco entero. _Evita_: filtro
