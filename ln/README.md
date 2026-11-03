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
- `fuente` lleva el PDF y la página de la diapositiva. Cada pregunta sale del temario, y no de otra fuente.
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

## El servidor: `serve.py`

`serve.py` usa solo la stdlib de Python. Sirve la interfaz y el banco, y tiene 2 rutas:

- `GET /api/respuestas` devuelve el historial entero.
- `POST /api/respuestas` añade una lista de respuestas al historial.

La interfaz guarda en `localStorage` cada respuesta sin enviar, y la envía cuando el servidor responde. Así, una sesión en el móvil sin red no pierde respuestas.

El servidor escucha en `127.0.0.1:8601`. `tailscale serve` pone HTTPS delante, en el puerto 8600 de la tailnet.
