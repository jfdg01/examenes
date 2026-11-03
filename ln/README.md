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
