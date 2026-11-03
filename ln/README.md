# LN: el test de práctica

Un banco de preguntas tipo test del temario de LN, con una interfaz web y un servidor que sincroniza el historial entre el móvil y el PC. El examen es el 20 de noviembre de 2026.

## El banco: `preguntas.json`

El banco es una lista de preguntas en JSON. El navegador lo lee con `fetch`, sin dependencias.

```json
{
  "id": "rgpd-001",
  "concepto": "rgpd-brecha-plazo",
  "tema": "rgpd",
  "fuente": { "pdf": "1_Legislacion.pdf", "pagina": 34 },
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
- Cada distractor viene de una confusión real del temario, como responsable y encargado.
- `explicacion` dice el contraste clave en una línea.

El banco tiene unas 250 preguntas: unas 170 de RGPD, ISO 27001, ENS y riesgos, y unas 80 del resto. Las preguntas tienen 4 opciones, aunque el examen sea de verdadero o falso. Una pregunta de 4 opciones entrena la diferencia entre ideas parecidas, y un examen de verdadero o falso pregunta esas mismas diferencias.

## El historial: `respuestas.jsonl`

El historial es una lista de respuestas que solo crece, con una respuesta por línea:

```json
{"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": true, "seguro": false}
```

La interfaz calcula todo el estado con esta lista: la caja de cada pregunta y el dominio de cada concepto. Dos dispositivos no tienen conflictos, porque cada uno solo añade líneas.

El historial no entra en git. El servidor lo guarda en `ln/respuestas.jsonl`.

## La confianza y las cajas

Antes de ver la solución, marcas "seguro" o "dudo". La pregunta va a una de 3 cajas:

| Respuesta | Caja siguiente |
|---|---|
| Fallo, o acierto con "dudo" | Caja 1 |
| Acierto con "seguro" | Una caja más, hasta la 3 |

La caja 1 sale en cada sesión, la caja 2 cada 2 días y la caja 3 cada 4 días. Una pregunta nueva empieza en la caja 1. Una sesión tiene 25 preguntas de temas mezclados, y las preguntas de la caja 1 van primero. Un fallo con "seguro" se marca en la pantalla de la solución, porque es una idea falsa.

La interfaz muestra el porcentaje de aciertos con "dudo". El examen resta entre 1/3 y 1/2 de un acierto por cada fallo. Con un porcentaje sobre el 33 %, una pregunta dudosa se contesta.

## La pantalla de la pregunta

Cada opción es una fila con 2 zonas. Un toque en el texto responde con "seguro", y un toque en el botón "dudo" de la derecha responde con "dudo". Un solo toque responde.

La solución sube desde abajo, en una hoja. Las opciones quedan detrás, en verde la correcta y en rojo la tuya si fallas. La hoja tiene, en este orden:

1. El aviso de un fallo con "seguro" o de un acierto con "dudo".
2. La `explicacion`.
3. El PDF, la página y el concepto.
4. "Ver la página aquí": la imagen de la página, dentro de la hoja.
5. "Abrir el PDF en la página N": el PDF entero, con `#page=N` en la URL.
6. El botón "Siguiente".

Las imágenes están en `ln/paginas/<pdf sin .pdf>-<pagina>.png`. `pdftoppm -png -r 80` las saca de los PDFs una sola vez. Las imágenes y los PDFs no entran en git.

Una sesión sin preguntas pendientes muestra la fecha de la próxima pregunta, y ofrece el simulacro. No hay un botón para reiniciar el historial, porque el historial guarda los fallos con "seguro" y el porcentaje de "dudo".

## El simulacro

El simulacro saca 40 preguntas al azar de todo el banco, y no mira las cajas. Es el repaso de los días 18 y 19 de noviembre.

Cada pregunta del simulacro tiene un botón "en blanco" debajo de las opciones. Al final, la interfaz muestra la nota con una penalización de 1/3 y con una de 1/2.

Las respuestas del simulacro entran al historial como las demás, porque son respuestas reales. Una pregunta en blanco cuenta en el historial como un fallo con "dudo".

## El servidor: `serve.py`

`serve.py` usa solo la stdlib de Python. Sirve la interfaz, el banco, los PDFs y las imágenes de las páginas, y tiene 2 rutas:

- `GET /api/respuestas` devuelve el historial entero.
- `POST /api/respuestas` añade una lista de respuestas al historial.

La interfaz guarda en `localStorage` cada respuesta sin enviar, y la envía cuando el servidor responde. Así, una sesión en el móvil sin red no pierde respuestas.

El servidor escucha en `127.0.0.1:8601`. `tailscale serve` pone HTTPS delante, en el puerto 8600 de la tailnet.

## Las 2 partes

El test tiene 2 partes independientes: la interfaz y el banco. El formato de `preguntas.json` es el único contrato entre las 2 partes. Una parte no cambia los archivos de la otra.

### La interfaz

La interfaz tiene el servidor, la página web, las cajas, el simulacro y las imágenes de las páginas.

- La página lee `preguntas.json`, y no tiene ninguna pregunta en su código.
- Los tests de la interfaz usan `ln/preguntas-ejemplo.json`, con 6 preguntas reales del temario. Los tests no dependen del banco real.
- Un script saca las imágenes de `ln/paginas/` de los 5 PDFs. El script no repite una imagen que ya existe.

### El banco

El banco tiene `ln/preguntas.json` y `ln/comprobar_banco.py`.

`comprobar_banco.py` usa solo la stdlib de Python, y falla con cada pregunta que rompe el formato:

- un `id` repetido, o un `tema` fuera de la lista;
- un número de distractores distinto de 3, o un distractor igual a la respuesta correcta;
- un `pdf` que no existe en `ln/`, o una `pagina` mayor que el número de páginas del PDF.

El banco crece por tandas, con una tanda por tema y un commit por tanda. La primera tanda es la del RGPD. Su revisión va antes de las demás tandas, porque fija el nivel de las preguntas y de los distractores.
