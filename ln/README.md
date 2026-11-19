# LN: el test de práctica

Un banco de preguntas tipo test del temario de LN, con una interfaz web y un servidor que sincroniza el historial entre el móvil y el PC. El examen es el 20 de noviembre de 2026, a las 16:30. Dura 25 minutos y tiene 20 preguntas tipo test, con 4 opciones como máximo. Un acierto suma 0,5 puntos, un fallo resta 0,125 y una pregunta en blanco no cuenta.

## El banco: `preguntas.json`

El banco es una lista de preguntas en JSON. El navegador lo lee con `fetch`, sin dependencias.

```json
{
  "id": "rgpd-001",
  "concepto": "rgpd-brecha-plazo",
  "tema": "rgpd",
  "fuente": { "pdf": "1_Legislacion.pdf", "pagina": 38 },
  "pregunta": "¿En qué plazo notifica el responsable una brecha a la autoridad de control?",
  "correcta": "72 horas desde que tiene constancia de la brecha",
  "distractores": ["24 horas desde que ocurre la brecha", "Un mes desde que tiene constancia", "Sin plazo fijo"],
  "explicacion": "Art. 33 del RGPD."
}
```

- `id` no cambia nunca, porque el historial lo usa como clave.
- `concepto` agrupa 2 o 3 preguntas que preguntan el mismo dato desde ángulos distintos. Así, una respuesta mide el dato, y no la forma de la pregunta.
- `tema` es uno de estos: `rgpd`, `iso27001`, `ens`, `riesgos`, `auditoria`, `lssi`, `software`, `propiedad`, `delitos`, `etica`, `certificaciones`.
- `fuente` lleva el PDF y la página del PDF. Cada página tiene 2 diapositivas, y `pagina` no es el número de la diapositiva. Cada pregunta sale del temario, y no de otra fuente.
- `correcta` va aparte de los `distractores`, y la interfaz baraja las 4 opciones. Una opción no dice "todas las anteriores", porque el barajado la rompe.
- `explicacion` dice el contraste clave en una línea.

El banco tiene unas 355 preguntas: unas 245 de RGPD, ISO 27001, ENS y riesgos, y unas 110 del resto. Las preguntas tienen 4 opciones, el máximo del examen. Una pregunta de 4 opciones entrena la diferencia entre ideas parecidas.

### La calidad de una pregunta

Las reglas siguen la revisión de Haladyna, Downing y Rodriguez (2002), "A review of multiple-choice item-writing guidelines", y la guía de redacción de preguntas del NBME, de Case y Swanson.

1. **Una sola respuesta defendible.** La correcta es cierta según su página, y cada distractor es falso según el temario. Un distractor "menos correcto" no vale. Un dato que el temario da de 2 formas distintas no tiene pregunta.
2. **El enunciado se contesta sin las opciones.** El enunciado nombra el concepto del temario y pide una sola cosa de ese concepto. Con las opciones tapadas, quien sabe el dato da una sola respuesta corta. Así, la pregunta entrena el recuerdo del dato, y no solo su reconocimiento. Un caso práctico pide un solo salto: del caso al concepto, o del concepto a lo que hace, y no los 2. El enunciado no lleva un detalle que la respuesta no necesita. El enunciado dice la idea con otras palabras, y no copia la frase del temario, pero usa el término del temario para el concepto.
3. **Ninguna pista fuera del dato.** Las 4 opciones tienen una longitud parecida, y la correcta es la más larga en un 25 % de las preguntas como mucho. Las 4 tienen la forma gramatical que pide el enunciado. Una palabra del enunciado no se repite solo en la correcta, y "siempre", "nunca" o "solo" no van solo en los distractores. Una sigla o una traducción del enunciado también es una pista: "CWNP" en el enunciado señala "CWSP", y "seguridad ofensiva" señala "Offensive Security". Ninguna opción es "todas las anteriores" ni "ninguna de las anteriores", porque la interfaz baraja las opciones.
4. **Cada distractor sale del temario que dio el profesor.** Un distractor es un nombre, una cifra o una idea del temario, fuera de los ejercicios y de las actividades. El mejor distractor es la confusión más probable de quien estudia, como responsable y encargado. Un nombre que no sale en el temario se descarta sin saber el dato.
5. **Ningún enunciado negativo.** Un "¿Cuál NO es…?" se lee mal con prisa, y entrena la búsqueda de la trampa, y no el dato.
6. **Una idea, y no un dato suelto.** La pregunta pide una idea del temario: una regla, un plazo, la diferencia entre 2 figuras o lo que hace algo. Una fecha del calendario, el número de una norma o el nombre de la organización que ofrece una certificación no tienen pregunta. Un plazo sí la tiene, porque es una regla, como las 72 horas para notificar una brecha. Una pregunta puede nombrar una norma por su número, pero no pide ese número.

Una pregunta que rompe la regla 6 deja su sitio a una pregunta nueva, sobre otra idea de la misma página. La pregunta nueva lleva un `id` nuevo, porque el historial del `id` viejo mide el dato suelto. Una página sin otra idea pierde la pregunta. Una pregunta que rompe la regla 3 conserva su `id`, porque su dato no cambia.

### Las preguntas difíciles

Una pregunta de cada 5 es difícil. Su distractor está muy cerca del dato, y falla solo en un detalle. Así, la pregunta obliga a entender el dato entero, y no solo a reconocer una palabra. Una pregunta difícil engaña con el contenido, y nunca con la forma: sigue las 6 reglas de calidad.

Un distractor difícil tiene una de estas formas:

- **Un solo cambio.** Copia la correcta y cambia un término: "responsable" por "encargado", o "desde que tiene constancia" por "desde que ocurre".
- **Una verdad que no responde.** Es cierto en el temario, pero responde a otra pregunta, como el plazo de otro artículo.
- **La excepción.** El caso parece seguir la regla general, pero el temario tiene una excepción para ese caso.
- **La cifra en otra unidad.** El número es el correcto, pero la unidad o la referencia cambian: "72 días", o el 4 % del beneficio.

Una pregunta pasa a ser difícil con otros distractores, y con el mismo dato. Así, su `id` mide el mismo dato en el historial.

## El historial: `respuestas.jsonl`

El historial es una lista de respuestas que solo crece, con una respuesta por línea:

```json
{"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": true, "seguro": false}
```

Una pregunta en blanco lleva además `"blanco": true`, con `acierto` y `seguro` en `false`. La clave `blanco` solo sale en un blanco. La razón está en [el ADR 0001](docs/adr/0001-el-blanco-lleva-su-propia-clave.md).

```json
{"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": false, "seguro": false, "blanco": true}
```

Un blanco anterior a esa clave tiene la forma de un fallo con "dudo", y queda así.

La interfaz calcula todo el estado con esta lista: la caja de cada concepto, su estado y las tasas de la tarjeta de la nota. Dos dispositivos no tienen conflictos, porque cada uno solo añade líneas.

El historial no entra en git. El servidor lo guarda en `ln/respuestas.jsonl`.

Una respuesta huérfana tiene un `id` que no está en el banco. Una pregunta que la regla 6 cambia deja sus respuestas huérfanas. La interfaz no usa las respuestas huérfanas en ninguna cuenta, y el archivo las conserva, porque una línea borrada no tiene vuelta atrás.

## La confianza y las cajas

Antes de ver la solución, marcas "seguro" o "dudo", o dejas la pregunta en blanco. El concepto de la pregunta va a una de 3 cajas:

| Respuesta | Caja siguiente |
|---|---|
| Fallo, acierto con "dudo" o blanco | Caja 1 |
| Acierto con "seguro" | Una caja más, hasta la 3 |

La caja es del concepto, y no de la pregunta. Una pregunta hermana es otra pregunta del mismo concepto. Cada vez que un concepto sale, sale con la pregunta que lleva más tiempo sin salir. Así, un concepto de 2 preguntas alterna entre las 2, y un concepto de 3 preguntas las recorre en orden. Una pregunta repetida mide el recuerdo de esa pregunta, y una pregunta hermana mide el concepto.

Cada concepto tiene uno de 3 estados. La última respuesta a una pregunta del concepto decide el estado:

| Estado | La regla |
|---|---|
| Lo sabes | La última respuesta es un acierto con "seguro" |
| No lo sabes | La última respuesta es un fallo, un acierto con "dudo" o un blanco |
| Sin ver | Ninguna pregunta del concepto tiene una respuesta |

Un acierto con "seguro" en una pregunta basta para saber el concepto, porque sus preguntas preguntan el mismo dato. Un fallo posterior en otra pregunta del concepto muestra una idea falsa, y el concepto pasa a "no lo sabes".

Una sesión tiene 12 preguntas, y llena sus huecos en este orden:

1. Los conceptos de la caja 1.
2. Los conceptos sin ver.
3. Un sorteo sin repetición de los otros conceptos, para los huecos que quedan.

Cada concepto sale una sola vez en una sesión. El paso 2 da la vuelta a todo el banco antes del repaso, porque un concepto nuevo enseña un dato. Con todos los conceptos vistos, el paso 2 queda vacío.

En el sorteo, un concepto de la caja 2 pesa 1, y un concepto de la caja 3 pesa 0,5. Así, un grupo grande sale más, y todo el banco avanza a la vez. Las 12 preguntas salen en un orden al azar, con los temas y las cajas mezclados.

Un bloque de estudio termina con una pausa de 3 horas sin respuestas. Un concepto de la caja 2 o de la caja 3 no sale en el bloque de su última respuesta. La caja 1 sale en cada sesión. Una sesión sin preguntas dice que el repaso vuelve después de una pausa de 3 horas.

Un fallo con "seguro" se marca en la pantalla de la solución, porque es una idea falsa.

El examen resta 1/4 de un acierto por cada fallo. La tarjeta de la nota, en la pantalla de inicio, dice si una pregunta dudosa se contesta, y muestra el porcentaje de aciertos de cada estado.

## La pantalla de inicio

Debajo del título, la pantalla de inicio tiene una barra con 3 tramos, uno por cada estado de un concepto: verde para "lo sabes", rojo para "no lo sabes" y gris para "sin ver". El largo de cada tramo es su parte de los conceptos del banco. Debajo de la barra van las 3 cifras: "Sabes 79 · No sabes 6 · Sin ver 66, de 151 conceptos".

El tramo gris baja con cada sesión, hasta que la vuelta a todo el banco termina.

Debajo de las cifras va la tarjeta de la nota, la única tarjeta de la pantalla. Arriba dice "Sacar más de 5", con un slider que elige la nota, de 0,5 a 10, con pasos de 0,5. La nota empieza en 5, y queda en `localStorage`, en `ln-objetivo`. El cálculo cuenta la nota elegida o más.

La tarjeta tiene 3 partes, de arriba abajo:

1. **La flecha**: a la izquierda, la probabilidad de hoy, "80 %", con "con 81 conceptos" debajo. A la derecha, "95 %", con "con 98 conceptos" debajo. En el medio, una flecha con "≈4 sesiones". La palabra "probabilidad" no sale.
2. **La barra del camino**: un tramo verde para los conceptos que sabes, un tramo amarillo para los conceptos que faltan, y una marca en el 95 %. El largo de cada tramo es su parte de los conceptos del banco. La barra no tiene bloques.
3. **El consejo**: "Contesta las dudas · +9 puntos", o "Deja las dudas en blanco · +N puntos". Los puntos son la diferencia entre la probabilidad de las 2 formas, en puntos de porcentaje. Un botón "?" abre el desglose, un `<details>` que empieza cerrado. El desglose sigue abierto cuando el slider se mueve.

El desglose da los aciertos en una duda, con una fila por grupo de conceptos:

| Fila | Conceptos | Aciertos en una duda | Detalle |
|---|---|---|---|
| en lo que viste | no lo sabes | el porcentaje de aciertos con "dudo" | "32 de 68 «dudo»" |
| en lo que no viste | sin ver | 25 % | "a ciegas, 1 de 4" |
| de media | los 2 grupos | la media de las 2 filas | |

La media pesa cada porcentaje con los conceptos de su fila. Sin respuestas con "dudo", la primera fila usa el 25 % y el detalle "a ciegas, 1 de 4". Una pregunta de un concepto sin ver da menos pistas que una pregunta de un concepto visto, y por eso cada grupo tiene su porcentaje.

El cálculo supone que cada pregunta del examen cae en un concepto al azar del banco:

- Una pregunta de un concepto que sabes es un acierto.
- Con las dudas en blanco, una pregunta de otro concepto no cuenta.
- Con las dudas contestadas, una pregunta de otro concepto acierta con la media del desglose, y un fallo resta 1/4 de un acierto.

La tarjeta sigue la forma con la probabilidad más alta para la nota elegida: el consejo, la cifra de hoy y el camino. Con otra nota, la mejor forma puede cambiar.

El camino busca el número más pequeño de conceptos con una probabilidad del 95 % o más, con la mejor forma en cada número. El 95 % es fijo. Los conceptos que faltan son ese número menos los conceptos que sabes.

El ritmo sale de las 60 últimas respuestas, 5 sesiones de 12. El ritmo es la diferencia entre los conceptos que sabes hoy y los conceptos que sabías sin esas respuestas, dividida entre esas sesiones. Con menos de 60 respuestas, cuentan todas. Las sesiones son los conceptos que faltan divididos entre el ritmo, redondeadas hacia arriba.

- Con un 95 % o más hoy, la flecha muestra "✓", sin la cifra de la derecha ni el tramo amarillo.
- Con un ritmo de 0 o menos, la flecha muestra "? sesiones".

Las cifras son un límite optimista. El examen pregunta con otras palabras, y no elige sus preguntas al azar del temario. Un fallo con "seguro" en el examen no entra en el cálculo. El ritmo sale sobre todo de conceptos nuevos, y baja cuando la vuelta a todo el banco termina. Así, las sesiones para una nota alta son más que la cifra de la flecha.

## La pantalla de la pregunta

Cada opción es una fila con 2 zonas. Un toque en el texto responde con "seguro", y un toque en el botón "dudo" de la derecha responde con "dudo". Un solo toque responde.

Después de la respuesta, las opciones quedan en verde la correcta y en rojo la tuya si fallas. Cada una de las dos lleva al final de la fila un circulito: un tick en la correcta y una cruz en la tuya. Debajo de las opciones sale el aviso de un fallo con "seguro" o de un acierto con "dudo".

### La pantalla ancha

Una pantalla de 900 px o más tiene 2 columnas estáticas, sin animaciones: la pregunta a la izquierda y la solución a la derecha. La columna de la solución dice "Elige una opción" antes de la respuesta.

La solución tiene, en este orden:

1. La `explicacion`.
2. El botón "Siguiente".
3. Una fila con 2 botones fijos: "Ver la página aquí" a la izquierda y "La fuente" a la derecha.
4. Un panel debajo de la fila. "Ver la página aquí" muestra la imagen de la página. "La fuente" muestra el PDF, la página, el concepto y el enlace "Abrir el PDF en la página N", con `#page=N` en la URL.

Un solo panel está abierto a la vez. Un toque en el botón abierto cierra el panel. Los botones no cambian de sitio, y el panel va debajo de "Siguiente", así que un panel abierto no mueve el botón.

El botón con un icono de engranaje, a la derecha de la línea de arriba, muestra los tiradores: los 2 bordes y el separador. Cada zona mide como mínimo el 15 % de la pantalla. Las posiciones quedan en `localStorage`, en `ln-columnas`, y cada dispositivo tiene las suyas.

### La pantalla del móvil

Una pantalla de menos de 900 px es una columna con 4 partes: la línea de arriba, la barra de progreso, el cuerpo con su scroll y el pie fijo. No tiene zonas, ni tiradores, ni engranaje.

- La línea de arriba lleva el progreso, el botón ✕, el botón de las notas y el botón de copiar.
- El cuerpo lleva la pregunta y las opciones. Tras responder, lleva también el aviso, la explicación y el campo de notas. El campo de notas empieza escondido hasta que lo muestras con el botón de las notas.
- Antes de responder, el pie dice "Toca el texto si estás seguro. Toca «dudo» si dudas." El pie lleva también el botón "En blanco", en la sesión y en el simulacro.
- Tras responder, las opciones que no son la correcta ni la tuya se atenúan. Si el cuerpo no cabe en la pantalla, esas opciones se pliegan tras el botón "ver las otras N opciones".
- Tras responder, el pie tiene 2 botones: "pág. N" a la izquierda y "Siguiente". "pág. N" abre la fuente a pantalla completa: la imagen de la página, que un toque amplía al 200 %, y el enlace "PDF". La ✕ y el botón de atrás del móvil cierran la fuente.

Debajo de las opciones hay un campo de texto para las notas de la pregunta. El botón con un icono de libreta y lápiz, a la izquierda del botón de copiar, muestra y esconde el campo. El navegador guarda esa elección en `localStorage`, en `ln-notas-visibles`. El campo guarda la nota en el servidor 500 ms después de la última tecla, y la nota vuelve cada vez que sale esa pregunta. Una nota vacía la borra.

El botón con un icono de copiar está a la derecha del botón de las notas. El botón copia al portapapeles el enunciado de la pregunta y sus opciones, con una letra por opción, en el orden en que la pantalla las muestra. El texto no incluye la opción correcta ni la explicación.

Las imágenes están en `ln/paginas/<pdf sin .pdf>-<pagina>.png`. `pdftoppm -png -r 80` las saca de los PDFs una sola vez. Las imágenes y los PDFs no entran en git.

Una sesión sin preguntas pendientes muestra la fecha de la próxima pregunta, y ofrece el simulacro. No hay un botón para reiniciar el historial, porque el historial guarda los fallos con "seguro" y el porcentaje de "dudo".

## El simulacro

El simulacro copia los parámetros del examen: 20 preguntas al azar de todo el banco, sin mirar las cajas, y 25 minutos. Es el repaso de los días 18 y 19 de noviembre.

Un reloj en la cabecera cuenta atrás desde 25:00. Con el reloj en 0:00, el simulacro termina y cada pregunta sin contestar cuenta en blanco.

Cada pregunta del simulacro tiene un botón "en blanco" debajo de las opciones. Un acierto suma 0,5 puntos, un fallo resta 0,125 y un blanco no cuenta. Al final, la interfaz muestra la nota sobre 10.

Las respuestas del simulacro entran al historial como las demás, porque son respuestas reales. Una pregunta en blanco entra al historial con `"blanco": true`. Una pregunta sin contestar al final del reloj no entra al historial.

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

La línea de arriba de la pregunta lleva un botón ✕, a la izquierda del botón de las notas. El botón pregunta "¿Abandonar la sesión? Las respuestas quedan en el historial." con el diálogo del navegador, `confirm`. En un simulacro, la pregunta dice "el simulacro". Un "sí" guarda `"ronda": null` y abre el inicio.

Con 2 dispositivos abiertos a la vez, gana el último guardado. Un dispositivo no ve el guardado del otro hasta que recarga la página.

## El servidor: `serve.py`

`serve.py` usa solo la stdlib de Python. Sirve la interfaz, el banco, los PDFs y las imágenes de las páginas, y tiene 6 rutas:

- `GET /api/respuestas` devuelve el historial entero.
- `POST /api/respuestas` añade una lista de respuestas al historial.
- `GET /api/notas` devuelve las notas, un objeto con el `id` de la pregunta como clave.
- `POST /api/notas` guarda `{"id", "texto"}`. Un `texto` vacío borra la nota. Las notas viven en `ln/notas.json`, que no entra en git.
- `GET /api/ronda` devuelve el guardado de la ronda, o `{"guardada": 0, "ronda": null}` sin guardado.
- `POST /api/ronda` guarda un guardado de la ronda. Un guardado con otros campos o con otros tipos recibe un 400. El guardado vive en `ln/ronda.json`, que no entra en git.

La interfaz guarda en `localStorage` cada respuesta sin enviar, y la envía cuando el servidor responde. Así, una sesión en el móvil sin red no pierde respuestas.

El servidor escucha en `127.0.0.1:8601`. `tailscale serve` pone HTTPS delante, en el puerto 8600 de la tailnet.

`--banco` sirve otro archivo en `/preguntas.json`, y `--historial` guarda las respuestas en otro archivo. Una prueba con `preguntas-ejemplo.json` usa los 2, porque ese banco repite los `id` del banco real:

```bash
python3 serve.py --banco preguntas-ejemplo.json --historial /tmp/ln-respuestas-ejemplo.jsonl
```

## Las 2 partes

El test tiene 2 partes independientes: la interfaz y el banco. El formato de `preguntas.json` es el único contrato entre las 2 partes. Una parte no cambia los archivos de la otra.

### La interfaz

La interfaz tiene el servidor, la página web, las cajas, el simulacro y las imágenes de las páginas.

- La página lee `preguntas.json`, y no tiene ninguna pregunta en su código.
- Los tests de la interfaz usan `ln/preguntas-ejemplo.json`, con 6 preguntas reales del temario. Los tests no dependen del banco real.
- `sacar_paginas.py` saca las imágenes de `ln/paginas/` de los 5 PDFs. El script no repite una imagen que ya existe.

### El banco

El banco tiene `ln/preguntas.json` y `ln/comprobar_banco.py`.

`comprobar_banco.py` usa solo la stdlib de Python, y falla con cada pregunta que rompe el formato:

- un `id` repetido, o un `tema` fuera de la lista;
- un número de distractores distinto de 3, o un distractor igual a la respuesta correcta;
- un `pdf` que no existe en `ln/`, o una `pagina` mayor que el número de páginas del PDF;
- un `concepto` con menos de 2 preguntas o con más de 3.

Sin argumento, `comprobar_banco.py` lee `ln/preguntas.json`. Con una ruta, lee ese banco.

El script también lista los `id` de las respuestas huérfanas de `ln/respuestas.jsonl`, como un aviso que no falla. El aviso muestra un `id` cambiado por error, porque un `id` no cambia nunca.

El banco crece por tandas, con una tanda por tema y un commit por tanda. La primera tanda es la del RGPD. Su revisión va antes de las demás tandas, porque fija el nivel de las preguntas y de los distractores.

Después de las tandas, una revisión independiente compara cada pregunta con su página y con las reglas de calidad. Quien escribe una pregunta no ve sus propios errores, y por eso la revisión la hacen agentes que no escribieron el banco. Los revisores dan la lista de fallos, y un solo agente cambia `preguntas.json`.

### Los tests

Los tests corren en `ln/`, y necesitan los PDFs:

```bash
node --test
python3 -m unittest
```

`node --test` prueba `logica.js`, la lógica de las cajas, de la sesión, del simulacro y del guardado de la ronda, sin el DOM. `python3 -m unittest` prueba `serve.py`, `sacar_paginas.py` y `comprobar_banco.py`.
