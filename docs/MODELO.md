# El modelo de la práctica

La app tiene 2 máquinas de estados: el estado de una pregunta y la ronda. El estado se calcula desde `respuestas.jsonl`. La ronda a medias y la selección se guardan en `ronda.json` y en `localStorage`, y `nucleo` la restaura desde el banco.

## El estado de una pregunta

Cada pregunta tiene su estado, y una respuesta mueve solo el de su pregunta. El concepto no tiene estado: agrupa las preguntas para la sesión y el simulacro. Un estado por concepto no vale: un acierto en una pregunta fácil subiría lo que un fallo en una difícil baja, y la difícil volvería sin fin. La razón está en [el ADR 0003](adr/0003-el-estado-es-de-la-pregunta.md). El hueco sale de FSRS, el planificador de Anki, con el crate `fsrs` 7. La razón está en [el ADR 0004](adr/0004-el-hueco-sale-de-fsrs.md).

- Una pregunta sin respuestas está **sin ver**.
- Cada respuesta lleva una nota de FSRS. Un acierto con "seguro" es Good. Un fallo, un acierto con "dudo" y una pregunta en blanco son Again. Hard y Easy no se usan.
- FSRS calcula el **recuerdo** de cada pregunta vista desde sus respuestas: la probabilidad de acertarla en un momento dado.
- **Los pasos de aprendizaje.** Again lleva la pregunta al paso de 1 minuto. Good en el paso de 1 minuto la lleva al de 10 minutos, y Good en el de 10 minutos la gradúa. Una pregunta sin ver acertada con "seguro" empieza en el paso de 10 minutos. FSRS actualiza el recuerdo con cada respuesta, también dentro de los pasos.
- Una pregunta graduada vence cuando su recuerdo baja al 90 %, el recuerdo objetivo. El objetivo es una constante del núcleo. Un Again la devuelve al paso de 1 minuto.
- **El tope de la víspera.** Ningún hueco pasa de las 00:00 del día antes del examen. Una respuesta dada después de ese momento no lleva tope, y una pregunta de un examen sin fecha tampoco.
- Cada respuesta mueve el estado, la de una sesión y la de un simulacro.
- El estado sale del historial unido: el del servidor y las respuestas del dispositivo que aún no llegaron. Una respuesta con el mismo `id` y `ts` cuenta una vez.

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

La sesión lleva como mucho una pregunta por concepto, porque una hermana (otra pregunta del mismo concepto) da pistas de la respuesta. La sesión recorre las preguntas en este orden, y toma la primera de cada concepto hasta llenar 12:

1. Las falladas vencidas, del fallo más viejo al más nuevo. Con falladas de 12 conceptos o más, la sesión no lleva nada más: un fallo se corrige antes de avanzar en el banco.
2. Las demás vencidas, de menor a mayor recuerdo.
3. Las hermanas sin ver de un concepto con alguna pregunta vista. Así, una idea se cierra antes de abrir otra.
4. Las preguntas de los conceptos sin ver.
5. Las preguntas que vencen antes.

Un concepto con una fallada y una vencida entra con la fallada, y la vencida espera a la sesión siguiente. Un fallo dentro de una sesión no vuelve en la misma sesión, y abre la sesión siguiente cuando pasa su minuto.

Dentro de un grupo, el azar rompe los empates. Las 12 preguntas salen en un orden al azar, y las opciones de cada una también. El azar sale de una semilla, y la misma semilla da el mismo orden.

## El simulacro

El simulacro lleva las preguntas del examen, al azar de la selección, con una pregunta por concepto. Solo rellena con hermanas cuando la selección tiene menos conceptos que preguntas el examen.
