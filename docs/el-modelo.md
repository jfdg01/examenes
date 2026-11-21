# El modelo de la práctica

La app tiene 3 máquinas de estados: la caja de una pregunta, el estado de un concepto y la ronda. Las 3 se calculan desde `respuestas.jsonl`, y ningún estado se guarda aparte. Este doc describe la lógica de `test/logica.js` sin el bloque de estudio. La revisión de la lógica, en [el registro de la migración](migracion-a-rust.md), decide qué cambia.

## La caja de una pregunta

- Una pregunta sin respuestas es **nueva**. El dominio la cuenta como caja 1.
- Un acierto con "seguro" sube una caja: de nueva a 2, de 1 a 2 y de 2 a 3. La caja 3 es el tope.
- Un fallo, un acierto con "dudo" o una pregunta en blanco bajan la pregunta a la **caja 1**.
- Cada respuesta mueve la caja, la de una sesión y la de un simulacro.
- Rust tiene un `enum Caja { Nueva, Una, Dos, Tres }`, y una función que da la caja siguiente según la respuesta.

## El estado de un concepto

- Un concepto sin respuestas está **sin ver**.
- La última respuesta a una pregunta del concepto decide su estado. Un acierto con "seguro" deja el concepto en **sabes**. Otra respuesta lo deja en **no sabes**.
- La barra del camino y la tarjeta de la nota usan este estado.

El **dominio** de un concepto es la caja más baja de sus preguntas. El PC muestra el dominio al lado de la pregunta. El estado y el dominio pueden no coincidir: un concepto con 2 preguntas, una acertada con "seguro" y otra nueva, está en "sabes" con el dominio en la caja 1.

## La ronda

- Del **inicio**, una ronda empieza como una sesión o como un simulacro.
- Cada pregunta pasa de **sin contestar** a **contestada**, y la pantalla muestra la solución. El simulacro añade el botón "En blanco" y el reloj.
- La última pregunta lleva al **final**, y el final vuelve al inicio. El botón ✕ también vuelve al inicio.
- Un simulacro con el `limite` pasado va al final, y cada pregunta sin contestar cuenta en blanco.
- La ronda a medias vive en `localStorage` y en el servidor. Al cargar, gana el guardado más nuevo.

## La sesión de 12 preguntas

La sesión junta las preguntas en este orden:

1. Todas las preguntas de la caja 1.
2. Una pregunta de cada concepto sin ver: la vuelta al banco.
3. Un sorteo entre las demás preguntas de los conceptos vistos. Cada caja tiene un peso en el sorteo.

La lista se corta en 12 preguntas, y sale en un orden al azar. Con todos los conceptos vistos, el sorteo siempre tiene candidatas. Así, una sesión de un banco de 12 preguntas o más tiene 12 preguntas. Una pregunta de la caja 3 puede salir en la sesión siguiente a su acierto.

Los pesos del sorteo salen del prototipo de la sesión. El prototipo simula muchas sesiones, y cuenta las preguntas nuevas y las de cada caja en cada sesión de 12. Los pesos dan un reparto sensato en cada sesión.
