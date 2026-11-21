# El test de práctica

Un banco de preguntas tipo test del temario de un examen, con una interfaz web y un servidor que sincroniza el historial entre el móvil y el PC. Esta carpeta tiene el código, y cada examen tiene una carpeta de datos al lado: `ln/`, `asn/` y `ps/`.

## Los exámenes

La carpeta de un examen tiene estos archivos:

- `examen.json`: las cifras del examen.
- `preguntas.json`: el banco.
- `pdfs/`: los PDFs del temario. `fuente.pdf` lleva solo el nombre del archivo.
- `paginas/`: las imágenes de las páginas de los PDFs.
- `README.md`: la fecha, el formato y el estado del banco de ese examen.

El historial, las notas y la ronda a medias también viven en la carpeta del examen. Los PDFs, las imágenes, el historial, las notas y la ronda no entran en git.

`examen.json` tiene estos campos:

```json
{
  "titulo": "ASN",
  "puerto": 8605,
  "preguntas": 35,
  "opciones": 3,
  "minutos": 45,
  "penalizacion": 0,
  "temas": ["introduccion", "ciclo-de-vida"]
}
```

- `puerto` es el puerto local del servidor. `tailscale serve` pone HTTPS delante, en el puerto anterior: 8604 para 8605.
- `preguntas` y `minutos` son las cifras del simulacro.
- `opciones` es el número de opciones de una pregunta del examen. Una pregunta del banco tiene `opciones` − 1 distractores.
- `penalizacion` es la parte de un acierto que un fallo resta: 0,25 resta 1/4 de un acierto, y 0 no resta.
- `temas` es la lista de los valores de `tema`.

Cada examen tiene su puerto, y el navegador separa el `localStorage` de cada puerto. Así, las claves `ln-*` de un examen no pisan las claves de otro examen.

Un examen nuevo es una carpeta nueva con su `examen.json`, un `preguntas.json` con `[]` y sus PDFs en `pdfs/`. Elige un `puerto` libre: `tailscale serve status` lista los puertos ocupados.

## El banco

El formato de `preguntas.json`, las reglas de una pregunta y el comprobador están en `BANCO.md`.

## El historial: `respuestas.jsonl`

El historial es una lista de respuestas que solo crece, con una respuesta por línea:

```json
{"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": true, "seguro": false}
```

La interfaz calcula todo el estado con esta lista: el paso de cada concepto y su fecha de repaso. Dos dispositivos no tienen conflictos, porque cada uno solo añade líneas.

Una respuesta huérfana tiene un `id` que no está en el banco. Una pregunta que pasa a preguntar otra idea lleva un `id` nuevo, y deja sus respuestas huérfanas. La interfaz no usa las respuestas huérfanas en ninguna cuenta, y el archivo las conserva, porque una línea borrada no tiene vuelta atrás.

## La confianza y el paso del concepto

Antes de ver la solución, marcas "seguro" o "dudo". El historial, ordenado por `ts`, da a cada concepto un paso y una fecha de repaso:

| Respuesta a una pregunta del concepto | Paso siguiente | El concepto vence |
|---|---|---|
| Ninguna | Sin ver | |
| Acierto con "seguro" | Un paso más | En `ts` + el hueco del paso |
| Fallo, acierto con "dudo" o pregunta en blanco | Paso 0 | En el acto |

El hueco del paso N es 10 minutos × 3^(N − 1): 10 min, 30 min, 1 h 30 min, 4 h 30 min, 13 h 30 min, 40 h… Cada acierto con "seguro" sube el paso, aunque el concepto no haya vencido. Las respuestas del simulacro también mueven el paso.

Cada concepto tiene uno de 3 estados. La última respuesta a una pregunta del concepto decide el estado:

| Estado | La regla |
|---|---|
| Lo sabes | La última respuesta es un acierto con "seguro" |
| No lo sabes | La última respuesta es un fallo, o un acierto con "dudo" |
| Sin ver | Ninguna pregunta del concepto tiene una respuesta |

Un acierto con "seguro" en una pregunta basta para saber el concepto, porque sus preguntas preguntan el mismo dato. Un fallo posterior en otra pregunta del concepto muestra una idea falsa, y el concepto pasa a "no lo sabes".

Una sesión tiene 12 preguntas, una por concepto, y llena sus huecos en este orden:

1. Los conceptos vencidos, del vencido hace más tiempo al vencido hace menos. Mientras quedan conceptos sin ver, los vencidos llenan 8 huecos como mucho.
2. Los conceptos sin ver, al azar.
3. Los conceptos que vencen antes.

Los 4 huecos de los conceptos sin ver dan la vuelta a todo el banco mientras sigue el repaso, porque un concepto nuevo enseña un dato. Con todos los conceptos vistos, los vencidos llenan los 12 huecos.

De cada concepto sale una pregunta sin respuestas, al azar entre las de ese concepto. Si todas tienen respuestas, sale la de la respuesta más vieja. Así las preguntas hermanas se turnan. Las 12 preguntas salen en un orden al azar, con los temas mezclados.

El título de la pregunta muestra "nuevo" para un concepto sin ver, o "paso N". La línea de la fuente muestra el paso del concepto.

Un fallo con "seguro" se marca en la pantalla de la solución, porque es una idea falsa.

Un fallo resta la `penalizacion` de un acierto. La tarjeta de la nota, en la pantalla de inicio, dice si una pregunta dudosa se contesta, y muestra el porcentaje de aciertos con "dudo".

## La pantalla de inicio

Debajo del título, la pantalla de inicio tiene una barra con 3 tramos, uno por cada estado de un concepto: verde para "lo sabes", rojo para "no lo sabes" y gris para "sin ver". El largo de cada tramo es su parte de los conceptos del banco. Debajo de la barra van las 3 cifras: "Sabes 79 · No sabes 6 · Sin ver 66, de 151 conceptos".

Debajo va la tarjeta de la nota, con un slider que elige la nota, de 0,5 a 10. La nota empieza en 5, y queda en `localStorage`, en `ln-objetivo`. El cálculo cuenta la nota elegida o más.

La tarjeta tiene 3 partes, de arriba abajo:

1. **La flecha**: a la izquierda, la probabilidad de hoy, "80 %", con "con 81 conceptos" debajo. A la derecha, "95 %", con "con 98 conceptos" debajo. En el medio, una flecha con "≈4 sesiones".
2. **La barra del camino**: un tramo verde para los conceptos que sabes, un tramo amarillo para los conceptos que faltan, y una marca en el 95 %. El largo de cada tramo es su parte de los conceptos del banco.
3. **El consejo**: "Contesta las dudas · +9 puntos", o "Deja las dudas en blanco · +N puntos". Los puntos son la diferencia entre la probabilidad de las 2 formas, en puntos de porcentaje. Un botón "?" abre el desglose.

El desglose da los aciertos en una duda, con una fila por grupo de conceptos:

| Fila | Conceptos | Aciertos en una duda | Detalle |
|---|---|---|---|
| en lo que viste | no lo sabes | el porcentaje de aciertos con "dudo" | "32 de 68 «dudo»" |
| en lo que no viste | sin ver | 1 de `opciones` | "a ciegas, 1 de 3" |
| de media | los 2 grupos | la media de las 2 filas | |

La media pesa cada porcentaje con los conceptos de su fila. Sin respuestas con "dudo", la primera fila usa 1 de `opciones` y el detalle "a ciegas, 1 de 3". Una pregunta de un concepto sin ver da menos pistas que una pregunta de un concepto visto, y por eso cada grupo tiene su porcentaje.

El cálculo supone que cada pregunta del examen cae en un concepto al azar del banco:

- Una pregunta de un concepto que sabes es un acierto.
- Con las dudas en blanco, una pregunta de otro concepto no cuenta.
- Con las dudas contestadas, una pregunta de otro concepto acierta con la media del desglose, y un fallo resta la `penalizacion` de un acierto.

La tarjeta sigue la forma con la probabilidad más alta para la nota elegida: el consejo, la cifra de hoy y el camino. Con otra nota, la mejor forma puede cambiar.

El camino busca el número más pequeño de conceptos con una probabilidad del 95 % o más, con la mejor forma en cada número. El 95 % es fijo. Los conceptos que faltan son ese número menos los conceptos que sabes.

El ritmo sale de las 60 últimas respuestas, 5 sesiones de 12. El ritmo es la diferencia entre los conceptos que sabes hoy y los conceptos que sabías sin esas respuestas, dividida entre esas sesiones. Con menos de 60 respuestas, cuentan todas. Las sesiones son los conceptos que faltan divididos entre el ritmo, redondeadas hacia arriba.

- Con un 95 % o más hoy, la flecha muestra "✓", sin la cifra de la derecha ni el tramo amarillo.
- Con un ritmo de 0 o menos, la flecha muestra "? sesiones".

Las cifras son un límite optimista. El examen pregunta con otras palabras, y no elige sus preguntas al azar del temario. Un fallo con "seguro" en el examen no entra en el cálculo. El ritmo sale sobre todo de conceptos nuevos, y baja cuando la vuelta a todo el banco termina. Así, las sesiones para una nota alta son más que la cifra de la flecha.

## La pantalla de la pregunta

Cada opción es una fila con 2 zonas. Un toque en el texto responde con "seguro", y un toque en el botón "dudo" de la derecha responde con "dudo". Un solo toque responde.

Después de la respuesta, las opciones quedan en verde la correcta y en rojo la tuya si fallas. Debajo de las opciones sale el aviso de un fallo con "seguro" o de un acierto con "dudo".

### La pantalla ancha

Una pantalla de 900 px o más tiene 2 columnas estáticas: la pregunta a la izquierda y la solución a la derecha.

La solución tiene, en este orden:

1. La `explicacion`.
2. El botón "Siguiente".
3. Una fila con 2 botones fijos: "Ver la página aquí" a la izquierda y "La fuente" a la derecha.
4. Un panel debajo de la fila. "Ver la página aquí" muestra la imagen de la página. "La fuente" muestra el PDF, la página, el concepto y el enlace "Abrir el PDF en la página N", con `#page=N` en la URL.

Un solo panel está abierto a la vez. El panel va debajo de "Siguiente", así que un panel abierto no mueve el botón.

El engranaje muestra los tiradores de las columnas. Sus posiciones quedan en `localStorage`, en `ln-columnas`, y cada dispositivo tiene las suyas.

### La pantalla del móvil

Una pantalla de menos de 900 px es una columna: la línea de arriba, la barra de progreso, el cuerpo con su scroll y el pie fijo.

- El cuerpo lleva la pregunta y las opciones. Tras responder, lleva también el aviso, la explicación y el campo de notas.
- Antes de responder, el pie dice "Toca el texto si estás seguro. Toca «dudo» si dudas." En el simulacro, el pie lleva también el botón "En blanco".
- Tras responder, las opciones que no son la correcta ni la tuya se atenúan. Si el cuerpo no cabe en la pantalla, esas opciones se pliegan tras el botón "ver las otras N opciones".
- Tras responder, el pie tiene 2 botones: "pág. N" a la izquierda y "Siguiente". "pág. N" abre la fuente a pantalla completa: la imagen de la página, que un toque amplía al 200 %, y el enlace "PDF". La ✕ y el botón de atrás del móvil cierran la fuente.

Debajo de las opciones va el campo de las notas de la pregunta. El botón de las notas lo muestra y lo esconde, y `ln-notas-visibles` guarda esa elección. El campo guarda la nota en el servidor 500 ms después de la última tecla, y la nota vuelve cada vez que sale esa pregunta.

El botón de copiar copia el enunciado y las opciones, con una letra por opción, sin la correcta ni la explicación.

Con un banco no vacío, una sesión siempre tiene preguntas, porque los conceptos que vencen antes llenan los huecos. No hay un botón para reiniciar el historial, porque el historial guarda los fallos con "seguro" y el porcentaje de "dudo".

## El simulacro

El simulacro copia los parámetros del examen: `preguntas` preguntas al azar de todo el banco, sin mirar los pasos, y `minutos` minutos. Es el repaso de los últimos días.

Un reloj en la cabecera cuenta atrás desde los `minutos` del examen. Con el reloj en 0:00, el simulacro termina y cada pregunta sin contestar cuenta en blanco.

Cada pregunta del simulacro tiene un botón "en blanco" debajo de las opciones. Al final, la interfaz muestra la nota sobre 10. Un acierto suma 10 / `preguntas` puntos, un fallo resta la `penalizacion` de un acierto y un blanco no cuenta.

Las respuestas del simulacro entran al historial como las demás, porque son respuestas reales. Una pregunta en blanco cuenta en el historial como un fallo con "dudo".

## La ronda a medias

Una ronda es una sesión o un simulacro, desde la primera pregunta hasta la pantalla del final. Una ronda a medias sobrevive a una recarga de la página y a un cambio de dispositivo.

Un guardado tiene 2 campos: `guardada`, la hora del guardado en milisegundos, y `ronda`. Sin ronda a medias, `ronda` es `null`. Una ronda tiene 4 campos:

- `ids`: los `id` de sus preguntas, en orden.
- `resultados`: "acierto", "fallo" o "blanco", uno por pregunta contestada.
- `simulacro`: `true` en un simulacro.
- `limite`: la hora del final del simulacro en milisegundos, o `null` en una sesión.

La interfaz guarda la ronda al empezar y después de cada respuesta. La pantalla del final guarda `"ronda": null`. Cada guardado va al servidor y a `localStorage`, en `ln-ronda`.

Al cargar, la interfaz lee las 2 copias, y gana la copia con la `guardada` más nueva. Si gana la copia de `localStorage`, la interfaz la envía al servidor. Así, una ronda del móvil sin red llega al servidor con la red, y un final sin red no resucita la ronda del servidor.

Con una ronda a medias, la app abre su pregunta, sin pasar por el inicio. La pregunta actual es la primera sin resultado: una recarga después de una respuesta, y antes de "Siguiente", abre la pregunta siguiente. Las opciones se barajan otra vez en cada carga.

Una ronda con un `id` que no está en el banco se descarta, y la app abre el inicio.

El reloj del simulacro cuenta hasta `limite`, una hora fija, así que sigue con la página cerrada. Un simulacro con el `limite` pasado abre la pantalla del final, y cada pregunta sin contestar cuenta en blanco.

El botón ✕ abandona la ronda, con un `confirm`: guarda `"ronda": null` y abre el inicio. Las respuestas quedan en el historial.

Con 2 dispositivos abiertos a la vez, gana el último guardado. Un dispositivo no ve el guardado del otro hasta que recarga la página.

## El servidor: `serve.py`

`serve.py` usa solo la stdlib de Python. Sirve los archivos, y las rutas `/api/respuestas`, `/api/notas` y `/api/ronda` leen con `GET` y guardan con `POST` el historial, `notas.json` y `ronda.json`.

La interfaz guarda en `localStorage` cada respuesta sin enviar, y la envía cuando el servidor responde. Así, una sesión en el móvil sin red no pierde respuestas.

El servidor recibe la carpeta del examen, y escucha en `127.0.0.1`, en el `puerto` de su `examen.json`. Un archivo de la carpeta del examen gana a un archivo de `test/` con la misma ruta. Así, la carpeta del examen sirve sus datos, y `test/` sirve la interfaz:

```bash
python3 test/serve.py asn
tailscale serve --bg --https=8604 http://127.0.0.1:8605
```

`--banco` sirve otro archivo en `/preguntas.json`, y `--historial` guarda las respuestas en otro archivo. Una prueba con `preguntas-ejemplo.json` usa los 2, porque ese banco repite los `id` del banco real:

```bash
python3 test/serve.py ln --banco preguntas-ejemplo.json --historial /tmp/ln-respuestas-ejemplo.jsonl
```

## Las 2 partes

El test tiene 2 partes independientes: la interfaz y el banco. El formato de `preguntas.json` es el único contrato entre las 2 partes. Una parte no cambia los archivos de la otra.

### La interfaz

- Los tests de la interfaz usan `ln/preguntas-ejemplo.json`, con 6 preguntas reales del temario. Los tests no dependen del banco real.
- `python3 test/sacar_paginas.py asn` saca las imágenes de `asn/paginas/`, como `<pdf sin .pdf>-<pagina>.png`, de los PDFs de `asn/pdfs/`.

### El banco

El banco es el `preguntas.json` de cada examen, y `test/comprobar_banco.py` lo comprueba. `BANCO.md` lo describe.

### Los tests

Los tests corren en `test/`, y necesitan los PDFs de `ln/pdfs/`:

```bash
node --test
python3 -m unittest
```

`node --test` prueba `logica.js`, la lógica de los pasos, de la sesión, del simulacro y del guardado de la ronda, sin el DOM. `python3 -m unittest` prueba `serve.py`, `sacar_paginas.py` y `comprobar_banco.py`.
