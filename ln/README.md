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

El banco tiene unas 320 preguntas: unas 225 de RGPD, ISO 27001, ENS y riesgos, y unas 100 del resto. Las preguntas tienen 4 opciones, el máximo del examen. Una pregunta de 4 opciones entrena la diferencia entre ideas parecidas.

### La calidad de una pregunta

1. **Una sola respuesta defendible.** La correcta es cierta según su página, y cada distractor es falso según el temario. Un distractor "menos correcto" no vale. Un dato que el temario da de 2 formas distintas no tiene pregunta.
2. **El enunciado se contesta sin las opciones.** Con las opciones tapadas, el enunciado ya pide un dato concreto. Así, la pregunta entrena el recuerdo del dato, y no solo su reconocimiento.
3. **Ninguna pista fuera del dato.** Las 4 opciones tienen una longitud parecida, y la correcta es la más larga en un 25 % de las preguntas como mucho. Las 4 tienen la forma gramatical que pide el enunciado. Una palabra del enunciado no se repite solo en la correcta, y "siempre", "nunca" o "solo" no van solo en los distractores. Una sigla o una traducción del enunciado también es una pista: "CWNP" en el enunciado señala "CWSP", y "seguridad ofensiva" señala "Offensive Security".
4. **Cada distractor sale del temario que dio el profesor.** Un distractor es un nombre, una cifra o una idea del temario, fuera de los ejercicios y de las actividades, como responsable y encargado. Un nombre que no sale en el temario se descarta sin saber el dato.
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

La interfaz calcula todo el estado con esta lista: la caja de cada pregunta y el dominio de cada concepto. Dos dispositivos no tienen conflictos, porque cada uno solo añade líneas.

El historial no entra en git. El servidor lo guarda en `ln/respuestas.jsonl`.

Una respuesta huérfana tiene un `id` que no está en el banco. Una pregunta que la regla 6 cambia deja sus respuestas huérfanas. La interfaz no usa las respuestas huérfanas en ninguna cuenta, y el archivo las conserva, porque una línea borrada no tiene vuelta atrás.

## La confianza y las cajas

Antes de ver la solución, marcas "seguro" o "dudo". La pregunta va a una de 3 cajas:

| Respuesta | Caja siguiente |
|---|---|
| Fallo, o acierto con "dudo" | Caja 1 |
| Acierto con "seguro" | Una caja más, hasta la 3 |

Una sesión tiene 12 preguntas. Las preguntas de la caja 1 entran primero. Un sorteo sin repetición llena los huecos que quedan: una pregunta nueva y una pregunta de la caja 2 pesan 1, y una pregunta de la caja 3 pesa 0,5. Así, un grupo grande sale más, y todo el banco avanza a la vez. Las 12 preguntas salen en un orden al azar, con los temas y las cajas mezclados.

Un bloque de estudio termina con una pausa de 3 horas sin respuestas. Una pregunta de la caja 2 o de la caja 3 no sale en el bloque de su última respuesta. La caja 1 sale en cada sesión. Una sesión sin preguntas dice que el repaso vuelve después de una pausa de 3 horas.

Un fallo con "seguro" se marca en la pantalla de la solución, porque es una idea falsa.

La interfaz muestra el porcentaje de aciertos con "dudo". El examen resta 1/4 de un acierto por cada fallo. Con un porcentaje sobre el 20 %, una pregunta dudosa se contesta: a ese porcentaje, los aciertos esperados compensan los fallos.

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

- La línea de arriba lleva el progreso, el botón de las notas y el botón de copiar.
- El cuerpo lleva la pregunta y las opciones. Tras responder, lleva también el aviso, la explicación y el campo de notas. El campo de notas empieza escondido hasta que lo muestras con el botón de las notas.
- Antes de responder, el pie dice "Toca el texto si estás seguro. Toca «dudo» si dudas." En el simulacro, el pie lleva también el botón "En blanco".
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

Las respuestas del simulacro entran al historial como las demás, porque son respuestas reales. Una pregunta en blanco cuenta en el historial como un fallo con "dudo".

## El servidor: `serve.py`

`serve.py` usa solo la stdlib de Python. Sirve la interfaz, el banco, los PDFs y las imágenes de las páginas, y tiene 4 rutas:

- `GET /api/respuestas` devuelve el historial entero.
- `POST /api/respuestas` añade una lista de respuestas al historial.
- `GET /api/notas` devuelve las notas, un objeto con el `id` de la pregunta como clave.
- `POST /api/notas` guarda `{"id", "texto"}`. Un `texto` vacío borra la nota. Las notas viven en `ln/notas.json`, que no entra en git.

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

`node --test` prueba `logica.js`, la lógica de las cajas, de la sesión y del simulacro, sin el DOM. `python3 -m unittest` prueba `serve.py`, `sacar_paginas.py` y `comprobar_banco.py`.
