# El modelo de la práctica

La app tiene 2 máquinas de estados: el paso de un concepto y la ronda. El paso se calcula desde `respuestas.jsonl`. La ronda a medias y la selección se guardan en `ronda.json` y en `localStorage`, y `nucleo` la restaura desde el banco.

## El paso de un concepto

- Un concepto sin respuestas está **sin ver**.
- Un acierto con "seguro" sube el paso en 1, aunque el concepto no haya vencido. El concepto vence 10 minutos × 3^(paso − 1) después de la respuesta.
- Un fallo, un acierto con "dudo" o una pregunta en blanco ponen el paso a 0, y el concepto vence en el acto.
- Cada respuesta mueve el paso, la de una sesión y la de un simulacro.
- El paso sale del historial unido: el del servidor y las respuestas del dispositivo que aún no llegaron. Una respuesta con el mismo `id` y `ts` cuenta una vez.

Un concepto con un paso de 1 o más está en **sabes**. La barra del camino y la tarjeta de la nota cuentan los conceptos en "sabes". La pantalla de la pregunta muestra el paso del concepto.

Rust tiene un `enum Concepto { SinVer, Paso(u32) }`.

## La ronda

- Del **inicio**, una ronda empieza como una sesión o como un simulacro.
- El inicio tiene una casilla por tema, y una por apartado en un tema con varios. La **selección** son los pares (tema, apartado) marcados. La sesión, el simulacro y la tarjeta usan solo sus preguntas, y la tarjeta dice "con 3 de 5 temas". Sin nada marcado, los botones no empiezan.
- Cada pregunta pasa de **sin contestar** a **contestada**, y la pantalla muestra la solución. El simulacro añade el botón "En blanco" y el reloj, y solo marca la opción elegida hasta el final.
- La cuadrícula lleva a cualquier pregunta de la ronda. Una pregunta contestada no cambia de respuesta, y una pregunta saltada espera la suya.
- La ronda va al **final** cuando todas las preguntas están contestadas. Con alguna sin contestar, "Terminar" queda gris y lleva a la primera que falta. El final vuelve al inicio, y el botón ✕ también.
- El final muestra la cuadrícula en verde y rojo. Un número abre su pregunta corregida, y "Resultado" vuelve al final.
- Un simulacro con el `limite` pasado va al final. Cada pregunta sin contestar cuenta en blanco para la nota, y no entra en el historial.
- La ronda a medias vive en `localStorage` y en el servidor, con sus ids, un resultado o `null` por pregunta y el límite del simulacro. El guardado lleva también la selección: un apartado nuevo sale desmarcado, y sin selección guardada todo está marcado. Al cargar, gana el guardado más nuevo, y la ronda sigue en la primera pregunta sin resultado. Una ronda con un id que ya no está en el banco no se restaura.

Rust tiene un `enum Ronda { Inicio, Pregunta { contestada: bool }, Final }`.

## La sesión de 12 preguntas

La sesión llena sus 12 preguntas en este orden:

1. Los conceptos fallados, del fallo más viejo al más nuevo. Con 12 fallados o más, la sesión no lleva nada más: un fallo se corrige antes de avanzar en el banco.
2. Los conceptos vencidos con un paso de 1 o más, del vencido hace más tiempo al vencido hace menos.
3. Los conceptos sin ver.
4. Los conceptos que vencen antes.

Un fallo dentro de una sesión no vuelve en la misma sesión, y abre la sesión siguiente.

La sesión lleva una pregunta por concepto, y una pregunta sin respuestas sale antes que una con respuestas. Las 12 preguntas salen en un orden al azar, y las opciones de cada una también. El azar sale de una semilla, y la misma semilla da el mismo orden.
