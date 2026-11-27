# El modelo de la práctica

El banco de un examen son 3 archivos: `conceptos.json`, `preguntas.json` con las preguntas de test, y `desarrollo.json` con las de desarrollo. Las preguntas salen de los conceptos, por [el ADR 0005](adr/0005-el-concepto-es-una-entidad.md). La página de una pregunta de test es la de su concepto. Las preguntas de un examen congelado sin `conceptos.json` no tienen página.

La app tiene 2 máquinas de estados: el estado de una pregunta y la ronda. El estado se calcula desde `respuestas.jsonl` en el test, y desde `desarrollo.jsonl` en el desarrollo. La ronda a medias y la selección se guardan en `ronda.json` y en `localStorage`, y `nucleo` la restaura desde el banco.

## El estado de una pregunta

Cada pregunta tiene su estado, y una respuesta mueve solo el de su pregunta. El concepto no tiene estado: agrupa las preguntas para la sesión y el simulacro. La razón está en [el ADR 0003](adr/0003-el-estado-es-de-la-pregunta.md). El estado sale de FSRS, por [el ADR 0004](adr/0004-el-planificador-es-fsrs.md).

- Una pregunta sin respuestas está **sin ver**.
- El estado de una pregunta vista es su estabilidad, su dificultad y la hora de su última respuesta. De ahí sale su probabilidad de acierto en cada momento.
- Un fallo y una pregunta en blanco son Again, un acierto con "dudo" es Hard, y un acierto con "seguro" es Good, también en una pregunta sin ver.
- Una pregunta vence cuando su probabilidad de acierto baja de la retención de `examen.json`, 0.9 si falta. Una fallada vence siempre, hasta que la aciertas.
- Cada respuesta mueve el estado, la de una sesión y la de un simulacro.
- El estado sale del historial unido: el del servidor y las respuestas del dispositivo que aún no llegaron. Una respuesta con el mismo `id` y `ts` cuenta una vez.

Una pregunta vista y no vencida está en **sabes**. La barra del camino, la tarjeta de la nota y el inicio cuentan preguntas. La pantalla de la pregunta muestra su probabilidad de acierto, y la solución dice cuándo vuelve.

## La ronda

- Del **inicio**, una ronda empieza como una sesión o como un simulacro.
- El inicio tiene una casilla por tema, y una por apartado en un tema con varios. La **selección** son los pares (tema, apartado) marcados. La sesión, el simulacro y la tarjeta usan solo sus preguntas, y la tarjeta dice "con 3 de 5 temas". Sin nada marcado, los botones no empiezan. Cada casilla dice cuántas de sus preguntas sabes, como "30/100 preguntas · 1/3 desarrollo", y cada tema lleva un enlace a su PDF. Una pregunta de desarrollo está en sabes con la misma regla que una de test, y solo cuenta en su línea propia: la tarjeta de la nota es solo del test. Los apartados de un tema empiezan plegados, y la tarjeta de los temas se pliega entera y lo recuerda, porque un examen con muchos apartados llena la pantalla.
- Cada pregunta pasa de **sin contestar** a **contestada**, y la pantalla muestra la solución. El simulacro añade el botón "En blanco" y el reloj, y solo marca la opción elegida hasta el final.
- Una respuesta de la sesión entra en el historial al contestar, y no cambia. Una respuesta del simulacro cambia hasta el final, como en el papel, y entra en el historial al terminar o al abandonar. Así el historial guarda solo la respuesta final.
- La cuadrícula lleva a cualquier pregunta de la ronda, y una pregunta saltada espera su respuesta.
- La ronda va al **final** cuando todas las preguntas están contestadas. Con alguna sin contestar, "Terminar" queda gris y lleva a la primera que falta. El final vuelve al inicio, y el botón ✕ también.
- El final muestra la cuadrícula en verde y rojo. Un número abre su pregunta corregida, y "Resultado" vuelve al final.
- Un simulacro con el `limite` pasado va al final. Cada pregunta sin contestar cuenta en blanco para la nota, y no entra en el historial.
- La ronda a medias vive en `localStorage` y en el servidor, con sus ids, un resultado o `null` por pregunta, las respuestas del simulacro que aún no están en el historial y el límite del simulacro. El guardado lleva también la selección: un apartado nuevo sale desmarcado, y sin selección guardada todo está marcado. Al cargar, gana el guardado más nuevo, y la ronda sigue en la primera pregunta sin resultado. Una ronda con un id que ya no está en el banco no se restaura.

Rust tiene un `enum Ronda { Inicio, Pregunta { contestada: bool }, Final }`.

## La sesión de 12 preguntas

La sesión lleva como mucho una pregunta por concepto, porque una hermana (otra pregunta del mismo concepto) da pistas de la respuesta. La sesión recorre las preguntas en este orden, y toma la primera de cada concepto hasta llenar 12:

1. Las preguntas falladas, del fallo más viejo al más nuevo. Con falladas de 12 conceptos o más, la sesión no lleva nada más: un fallo se corrige antes de avanzar en el banco.
2. Las demás preguntas vencidas, de la menor probabilidad de acierto a la mayor.
3. Las hermanas sin ver de un concepto con alguna pregunta vista. Así, una idea se cierra antes de abrir otra.
4. Las preguntas de los conceptos sin ver.
5. Las preguntas no vencidas, de la menor probabilidad de acierto a la mayor.

Un concepto con una fallada y una vencida entra con la fallada, y la vencida espera a la sesión siguiente. Un fallo dentro de una sesión no vuelve en la misma sesión, y abre la sesión siguiente.

Dentro de un grupo, el azar rompe los empates. Las 12 preguntas salen en un orden al azar, y las opciones de cada una también. El azar sale de una semilla, y la misma semilla da el mismo orden.

## El simulacro

El simulacro lleva las preguntas del examen, al azar de la selección, con una pregunta por concepto. Solo rellena con hermanas cuando la selección tiene menos conceptos que preguntas el examen.

Con `desarrollo` en `examen.json`, el simulacro es **mixto**: lleva primero esas preguntas de desarrollo, al azar de la selección, y luego el test, con un solo reloj. La cuadrícula numera las dos partes seguidas.

- Una pregunta de desarrollo lleva solo el enunciado y el cuadro de texto. Un cuadro vacío no bloquea "Terminar", y cuenta como Again.
- Al terminar, cada respuesta entra en el historial **sin corregir**, y un cuadro vacío entra con su Again. Al abandonar, entran solo las respuestas con texto.
- El final pone cada texto junto a la respuesta modelo y sus puntos, con los botones de la nota. Una respuesta sin clic queda sin corregir, y la corrige después el agente, como dice [`DESARROLLO.md`](DESARROLLO.md).
- La nota pesa cada parte con `pesos`. Con alguna respuesta sin corregir, la nota dice "desarrollo sin corregir". El test resta sus fallos sin límite en 0.

## La sesión de desarrollo

Una pregunta de desarrollo tiene su estado de FSRS, aparte del test, por [el ADR 0006](adr/0006-el-desarrollo-es-un-banco-aparte.md). Una nota de desarrollo no mueve ninguna pregunta de test.

- El botón "Desarrollo" va a la derecha de "Tipo test", y dice cuántas preguntas lleva su sesión, como "Desarrollo (4)". Sin `desarrollo.json`, no sale.
- Una pregunta de desarrollo está en la selección cuando su apartado está marcado en un tema que lleva el PDF de alguno de sus conceptos.
- La sesión lleva 4 preguntas: las falladas, de la más vieja a la más nueva; las vencidas, de la menor probabilidad de acierto a la mayor; las sin ver; y las no vencidas. No lleva la regla de un concepto por pregunta, porque cada pregunta de desarrollo cubre varios conceptos.
- "No la sabía", "A medias" y "La sabía" son Again, Hard y Good. "Ver respuesta modelo" funciona con el cuadro vacío, para una respuesta pensada y no escrita.
- La nota entra en el historial al ponerla, como en la sesión de test, y la sesión pasa a la siguiente sin hacer. Cuando todas están hechas, vuelve al inicio, sin final.
- "Anterior" y "Siguiente" saltan entre las 4. Una pregunta hecha enseña su texto, la respuesta modelo y su nota, sin cambiarlos, porque FSRS ya contó esa nota.
- "Corregir luego" sale entre las flechas, bajo las 3 notas, con la respuesta modelo abierta, porque al compararlas se ve si cuesta corregirla. Guarda el texto sin nota. Con el cuadro vacío queda gris.
- El inicio dice "N respuestas sin corregir" cuando el historial las tiene. Una respuesta sin corregir no mueve su FSRS hasta su corrección.
- La ronda guardada lleva el texto de cada pregunta y lo hecho con cada una, su nota o "Corregir luego", así que una recarga sigue en la primera sin hacer, con su texto. La del simulacro mixto lleva los textos del desarrollo, y vuelve al desarrollo si el test no tiene respuestas.
