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

**Acierto con "seguro"**: Una respuesta correcta, marcada como segura antes de ver la solución. Es la única respuesta que sube el paso.

**En blanco**: Una pregunta que dejas sin contestar a propósito. Entra en el historial, y pone el paso a 0 como un fallo. Una pregunta sin contestar al pasar el límite del simulacro no entra en el historial.

## El estado de una pregunta

**Sin ver**: Una pregunta sin ninguna respuesta registrada. _Evita_: nuevo

**Paso**: El número de aciertos con "seguro" seguidos de una pregunta. Cualquier otra respuesta lo pone a 0. _Evita_: caja, nivel, racha

**Hueco**: El tiempo entre una respuesta y el vencimiento de la pregunta. Crece con el paso. _Evita_: intervalo, pausa

**Vencida**: Una pregunta cuyo hueco ya pasó, y que toca repasar.

**Fallada**: Una pregunta vista con el paso a 0, por un fallo, un acierto con "dudo" o una respuesta en blanco. Una fallada está siempre vencida. _Evita_: caja, pendiente

**Sabes**: Una pregunta con un paso de 1 o más. _Evita_: dominado, sabido

## La práctica

**Ronda**: Una serie de preguntas de principio a fin: una sesión o un simulacro.

**Sesión**: Una ronda de 12 preguntas, como mucho una por concepto de la selección: primero las falladas, y después las demás por su vencimiento. _Evita_: repaso, bloque

**Simulacro**: Una ronda con el formato del examen: sus preguntas al azar de la selección, y su tiempo.

**Selección**: Los apartados de cada tema que marcas en el inicio. Las rondas y la tarjeta de la nota usan solo sus preguntas. Sin nada marcado antes, la selección es el banco entero. _Evita_: filtro
