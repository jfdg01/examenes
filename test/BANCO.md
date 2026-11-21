# El banco: `preguntas.json`

El banco de un examen es una lista de preguntas en JSON, en `<examen>/preguntas.json`. Es el único contrato entre el banco y la interfaz.

```json
{
  "id": "transporte-007",
  "concepto": "transporte-ssh-clave-de-sesion",
  "tema": "transporte",
  "fuente": { "pdf": "3_Transporte.pdf", "pagina": 41 },
  "pregunta": "En SSH, ¿cómo acuerdan el cliente y el servidor la clave de sesión?",
  "correcta": "Con un intercambio Diffie-Hellman, firmado por la clave del servidor",
  "distractores": ["El cliente la cifra con la clave pública del servidor", "El servidor la envía firmada con su clave privada"],
  "explicacion": "✓ SSH acuerda la clave con Diffie-Hellman, y el servidor firma el intercambio.\n✗ «La cifra con la clave pública del servidor»: es el intercambio RSA de TLS 1.2, y no el de SSH.\n✗ «La envía firmada»: una firma da autenticidad, y no confidencialidad. La clave viajaría a la vista."
}
```

- `id` es la clave del historial, y no cambia nunca. Empieza por el `tema`: `transporte-007`.
- `concepto` agrupa las preguntas de una misma idea.
- `tema` es uno de los temas del PDF de su `fuente` en el campo `pdfs` de `examen.json`.
- `fuente` es el PDF y su página de PDF, y no el número de la diapositiva. Es la página que explica la idea, y no una página que solo la nombra.
- `distractores` tiene `opciones` − 1 opciones. La interfaz baraja la correcta con los distractores.

## Las reglas

Las reglas siguen la revisión de Haladyna, Downing y Rodriguez (2002) y la guía de redacción del NBME. Valen para cada banco, salvo el de LN, que tiene 2 o 3 preguntas por concepto y una explicación de una línea.

1. **Un concepto es una idea del temario.** Una idea es una regla, un mecanismo, la diferencia entre 2 cosas o lo que hace algo. Una página da tantos conceptos como ideas tiene, de 0 a varios. Una fecha, un número de RFC o de norma, o el nombre de quien publica algo no son una idea.
2. **3 preguntas por concepto**, cada una desde un ángulo distinto: la definición, un caso, la consecuencia. Así, un acierto mide la idea, y no la forma de la pregunta.
3. **Una sola respuesta defendible.** La correcta es cierta según su página, y cada distractor es falso. Un dato que el temario da de 2 formas no tiene pregunta.
4. **El enunciado se contesta sin las opciones.** Con las opciones tapadas, quien sabe la idea da una respuesta corta. El enunciado pide una sola cosa, con el término del temario y con otras palabras que la diapositiva.
5. **Cada distractor es un error plausible**, la confusión más probable de quien estudia: responsable y encargado, firma y cifrado. Sale del temario, o es una idea plausible fuera de él. Cada distractor falla por un _matiz_ que la explicación nombra.
6. **Ninguna pista fuera de la idea.** Las opciones tienen una longitud y una forma gramatical parecidas. La correcta es la más larga en 1 de cada 3 preguntas como mucho. Una palabra, una sigla o una traducción del enunciado sale en todas las opciones o en ninguna. Un "siempre", un "nunca" o un "solo" que hace falso un distractor por exclusión lo delata, y sale también en la correcta. "Un solo" y "una sola" dan un número, y no son un absoluto. Cada opción se lee sola, porque la interfaz las baraja.
7. **El enunciado es afirmativo**, y pregunta qué es, qué hace o por qué.
8. **La explicación nombra cada matiz, con una línea por opción.** La primera línea empieza por `✓` y explica la correcta. Cada distractor tiene después su línea, en el orden de `distractores`. La línea empieza por `✗` y por las palabras clave del distractor entre comillas, y sigue con su matiz. Una letra no sirve, porque la interfaz baraja las opciones.
9. **Una sigla de seguridad es un concepto propio, con una sola pregunta:** el enunciado da la sigla y pide su significado, con 2 desarrollos plausibles como distractores. Las siglas generales de informática, como TCP, IP o ASCII, quedan fuera. Su `fuente` es la primera página que la explica.

Una pregunta de cada 5 es difícil: un distractor copia la correcta y cambia un solo término, responde a otra pregunta del temario, o aplica la regla general a su excepción. Una pregunta difícil sigue las 9 reglas.

Una pregunta que pasa a preguntar otra idea lleva un `id` nuevo. Una pregunta que cambia de forma o de distractores conserva su `id`.

El `README.md` de cada examen dice lo que el profesor marcó como importante. Esas ideas tienen su concepto.

## El comprobador

```bash
python3 test/comprobar_banco.py ps/preguntas.json
```

`comprobar_banco.py` falla con cada pregunta que rompe el formato: un `id` repetido, un `tema` que no es de su PDF, un número de distractores distinto de `opciones` − 1, un distractor igual a la correcta, un `pdf` o una `pagina` que no existen, y un `concepto` con menos de 2 preguntas o con más de 3. Un `concepto` que empieza por `sigla-` tiene una sola pregunta. También avisa, sin fallar, de 3 cosas:

- un "siempre", un "nunca" o un "solo" de un distractor que la correcta no dice;
- una correcta que es la opción más larga en más de 1 de cada 3 preguntas, sin contar las siglas;
- una respuesta del historial con un `id` fuera del banco.

Un aviso de una pista lo juzga el revisor: un absoluto que no delata al distractor se queda.

## La revisión

Una revisión de agentes que no escribieron las preguntas compara cada pregunta con su página y con las reglas. Cada revisor comprueba también que la `fuente` es la página que explica la idea. Los revisores no cambian `preguntas.json`: cada uno da un informe en JSON, con una lista de fallos.

```json
{"id": "transporte-007", "regla": 3, "gravedad": "grave", "fallo": "El distractor 2 también es cierto según la página 41.", "arreglo": {"distractores": ["…", "…"]}}
```

- `regla` es el número de la regla rota, y 0 para una `fuente` equivocada.
- `gravedad` es `grave` para un fallo de la regla 3, y `leve` para el resto.
- `arreglo` da los campos nuevos de la pregunta, o `null` para un fallo sin arreglo claro. Un arreglo cambia solo `pregunta`, `correcta`, `distractores`, `explicacion` y `fuente`.

Un solo agente aplica los informes:

```bash
python3 test/aplicar_revision.py ps/preguntas.json revision-*.json
```

El script lista los fallos sin aplicar, y ese agente los arregla a mano. Después corre el comprobador.
