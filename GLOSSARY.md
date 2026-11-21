# Los exámenes

Una app para practicar exámenes tipo test, con un banco de preguntas por examen.

## El examen y su banco

**Examen**: Una asignatura con su formato: el número de preguntas, de opciones y de minutos, y la penalización de un fallo. _Evita_: test, asignatura

**Banco**: Las preguntas de un examen.

**Concepto**: La idea que una pregunta pone a prueba. Varias preguntas del banco comparten un concepto. _Evita_: tema (un tema agrupa conceptos)

**Historial**: Todas las respuestas de un examen, en orden de hora.

## La respuesta

**Acierto con "seguro"**: Una respuesta correcta, marcada como segura antes de ver la solución. Es la única respuesta que sube el paso.

## El concepto

**Sin ver**: Un concepto sin ninguna respuesta registrada. _Evita_: nuevo

**Paso**: El número de aciertos con "seguro" seguidos de un concepto. Cualquier otra respuesta lo pone a 0. _Evita_: caja, nivel, racha

**Hueco**: El tiempo entre una respuesta y el vencimiento del concepto. Crece con el paso. _Evita_: intervalo, pausa

**Vencido**: Un concepto cuyo hueco ya pasó, y que toca repasar.

**Sabes**: Un concepto con un paso de 1 o más. _Evita_: dominado, sabido

## La práctica

**Ronda**: Una serie de preguntas de principio a fin: una sesión o un simulacro.

**Sesión**: Una ronda de 12 preguntas, una por concepto, elegidas por su vencimiento. _Evita_: repaso, bloque

**Simulacro**: Una ronda con el formato del examen: sus preguntas al azar de todo el banco, y su tiempo.
