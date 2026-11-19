# El blanco lleva su propia clave

Una pregunta en blanco entra en `respuestas.jsonl` con `"blanco": true`, y con `acierto` y `seguro` en `false`. La clave solo sale en un blanco. Así, una línea vieja conserva su significado, y una función que lee solo `acierto` y `seguro` trata un blanco como un fallo con "dudo": la caja 1 y el estado "no lo sabes". Un móvil con la versión vieja de `app.js` en caché sigue enviando líneas válidas.

## Las opciones descartadas

- `"acierto": null` y `"seguro": null`: cada función que lee esos campos necesita una revisión, y un `null` falla en silencio.
- Un campo `"resultado": "acierto" | "fallo" | "blanco"` en lugar de `acierto`: rompe cada línea del historial, o pide una migración.

## Las consecuencias

Un blanco anterior a la clave tiene la forma de un fallo con "dudo", y nada lo separa. Esos blancos cuentan como fallos con "dudo" en las tasas de la tarjeta de la nota.
