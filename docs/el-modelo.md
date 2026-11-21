# El modelo de la práctica

La app tiene 2 máquinas de estados: el paso de un concepto y la ronda. Las 2 se calculan desde `respuestas.jsonl`, y ningún estado se guarda aparte. El paso y la sesión entran antes en la app de JavaScript, con `los-intervalos-del-concepto` (#17), y la migración los pasa a Rust.

## El paso de un concepto

- Un concepto sin respuestas está **sin ver**.
- Un acierto con "seguro" sube el paso en 1, aunque el concepto no haya vencido. El concepto vence 10 minutos × 3^(paso − 1) después de la respuesta.
- Un fallo, un acierto con "dudo" o una pregunta en blanco ponen el paso a 0, y el concepto vence en el acto.
- Cada respuesta mueve el paso, la de una sesión y la de un simulacro.

Un concepto con un paso de 1 o más está en **sabes**. La barra del camino y la tarjeta de la nota cuentan los conceptos en "sabes". La pantalla de la pregunta muestra el paso del concepto.

Rust tiene un `enum Concepto { SinVer, Paso(u32) }`.

## La ronda

- Del **inicio**, una ronda empieza como una sesión o como un simulacro.
- Cada pregunta pasa de **sin contestar** a **contestada**, y la pantalla muestra la solución. El simulacro añade el botón "En blanco" y el reloj.
- La última pregunta lleva al **final**, y el final vuelve al inicio. El botón ✕ también vuelve al inicio.
- Un simulacro con el `limite` pasado va al final, y cada pregunta sin contestar cuenta en blanco.
- La ronda a medias vive en `localStorage` y en el servidor. Al cargar, gana el guardado más nuevo.

Rust tiene un `enum Ronda { Inicio, Pregunta { contestada: bool }, Final }`.

## La sesión de 12 preguntas

1. Los conceptos vencidos, del vencido hace más tiempo al vencido hace menos.
2. Mientras quedan conceptos sin ver, 4 huecos son para ellos.
3. Los huecos que sobran se llenan con más conceptos sin ver, y después con los conceptos que vencen antes.

La sesión lleva una pregunta por concepto, y una pregunta sin respuestas sale antes que una con respuestas. Las 12 preguntas salen en un orden al azar.
