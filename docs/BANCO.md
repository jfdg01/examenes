# El banco: `preguntas.json`

El banco de un examen es una lista de preguntas en JSON, en `<examen>/preguntas.json`. Es el único contrato entre el banco y la interfaz. `<examen>` es la carpeta del examen, como `examenes/ps`.

```json
{
  "id": "transporte-007",
  "concepto": "transporte-ssh-clave-de-sesion",
  "apartado": "transporte",
  "pregunta": "En SSH, ¿cómo acuerdan el cliente y el servidor la clave de sesión?",
  "correcta": "Con un intercambio Diffie-Hellman, firmado por la clave del servidor",
  "distractores": [
    "El cliente la cifra con la clave pública del servidor",
    "El servidor la envía firmada con su clave privada"
  ],
  "explicacion": "✓ Con Diffie-Hellman, los dos calculan la misma clave sin enviarla. La firma del servidor impide que un intermediario se haga pasar por él.\n✗ «La cifra con la clave pública del servidor»: es el intercambio RSA de TLS 1.2, y no el de SSH.\n✗ «La envía firmada»: una firma da autenticidad, y no confidencialidad. La clave viajaría a la vista."
}
```

- `id` es la clave del historial, y no cambia nunca. Empieza por el `apartado`: `transporte-007`, y `transporte-s03` para una sigla.
- `concepto` agrupa las preguntas de una misma idea, y es un `id` de `conceptos.json`. La página de la pregunta es la `fuente` de su concepto: el PDF y su página de PDF, y no el número de la diapositiva. Es la página que explica la idea, y no una página que solo la nombra.
- `apartado` es el de su concepto. En un examen congelado sin `conceptos.json`, es un apartado de un tema de `examen.json`. Su nombre está en el campo `apartados` de `examen.json`, y la cabecera de la pregunta lo muestra.
- `distractores` tiene `opciones` − 1 opciones. La interfaz baraja la correcta con los distractores.

## Las reglas

Las reglas siguen la revisión de Haladyna, Downing y Rodriguez (2002) y la guía de redacción del NBME. Valen para cada banco, salvo los congelados, como ASN y CA, que no se reescriben.

1. **Un concepto es una idea del temario.** Una idea es una regla, un mecanismo, la diferencia entre 2 cosas o lo que hace algo. Una página da tantos conceptos como ideas tiene, de 0 a varios. Una fecha, un número de RFC o de norma, o el nombre de quien publica algo no son una idea.
2. **3 preguntas por concepto**, cada una desde un ángulo distinto, lo más ortogonales entre sí que sea razonable. Así, los 3 ángulos cubren la idea entera, y la app repasa cada uno por separado.
3. **Una sola respuesta defendible.** La correcta es cierta según su página, y cada distractor es falso. Un dato que el temario da de 2 formas no tiene pregunta.
4. **El enunciado se contesta sin las opciones.** Con las opciones tapadas, quien sabe la idea da una respuesta corta. El enunciado pide una sola cosa, con el término del temario y con otras palabras que la diapositiva. El enunciado da su contexto: el protocolo, el mecanismo o el proceso del que habla. Quien sabe la idea y no ha visto la diapositiva entiende qué se pregunta. Una palabra con un sentido propio en la diapositiva lleva ese sentido: "el orden" no basta, y "el orden de cifrado y MAC para proteger la integridad" sí. Un ejemplo de la diapositiva sale descrito en el enunciado, con los datos que la respuesta necesita. "El protocolo de ejemplo" no basta. Un ejemplo que no cabe en el enunciado no tiene pregunta: la pregunta va a la idea general. El enunciado no remite al documento: "según el PDF" o "en la diapositiva" no van. Un nombre propio sale entero: "el Gran Cortafuegos chino", y no "el chino".
5. **Cada distractor es un error plausible**, la confusión más probable de quien estudia. Sale del temario, o es una idea plausible fuera de él. Cada distractor falla por un _matiz_ que la explicación nombra.
6. **Ninguna pista fuera de la idea.** Las opciones tienen una longitud y una forma gramatical parecidas. La correcta es la más larga en 1 de cada 3 preguntas de media. Un término clave, una sigla o una traducción del enunciado sale en todas las opciones o en ninguna. Una palabra común, como "atacante" o "servidor", queda fuera de esta regla. Un "siempre", un "nunca" o un "solo" que hace falso un distractor por exclusión lo delata, y sale también en la correcta. "Un solo" y "una sola" dan un número, y no son un absoluto. Cada opción se lee sola, porque la interfaz las randomiza.
7. **El enunciado es afirmativo**, y pregunta qué es, qué hace o por qué.
8. **La explicación nombra cada matiz, con una línea por opción.** La primera línea empieza por `✓` y dice por qué la correcta responde a la pregunta: el mecanismo o la razón que la hace cierta. No repite el enunciado ni la correcta en forma afirmativa, porque quien falló ya la lee en las opciones y no aprende nada nuevo. Cada distractor tiene después su línea, en el orden de `distractores`. La línea empieza por `✗` y por las palabras clave del distractor entre comillas, y sigue con su matiz. Una letra no sirve, porque la interfaz randomiza las opciones.
9. **Una sigla de seguridad es un concepto propio, con una sola pregunta:** el enunciado da la sigla y pide su significado, con 2 desarrollos plausibles como distractores. Las siglas generales de informática, como TCP, IP o ASCII, quedan fuera. La `fuente` de su concepto es la primera página que la explica.

Una pregunta que pasa a preguntar otra idea lleva un `id` nuevo. Una pregunta que cambia de forma o de distractores conserva su `id`, como en una edición a mano del JSON.

El `README.md` de cada examen dice lo que el profesor marcó como importante. Esas ideas tienen su concepto. Una idea marcada que ningún PDF trae cita la página que más se acerca, y su correcta es cierta según el `README.md`.

## El comprobador

```bash
cargo run --manifest-path app/Cargo.toml -p banco -- comprobar examenes/ps
```

`banco comprobar` recibe la carpeta del examen. Valida también `conceptos.json` y `desarrollo.json`. Falla sin `conceptos.json`, salvo en un examen congelado. Un apartado sin preguntas de test acepta conceptos sin preguntas. Falla con cada concepto o pregunta que rompe el formato: un `id` repetido, un concepto con un `apartado` que no es de su PDF o con un `pdf` o una `pagina` que no existen, una pregunta con un `concepto` que no está en `conceptos.json` o de otro apartado, un apartado sin nombre, un número de distractores distinto de `opciones` − 1, un distractor igual a la correcta, y un `concepto` con menos de 3 preguntas o con más de 3. Un `concepto` que empieza por `sigla-` tiene una sola pregunta. Falla también con una línea `✗` cuya clave `«…»` no está en su distractor, en el orden de `distractores`, sin distinguir mayúsculas, y con dos líneas `✗` de una pregunta con la misma clave. También avisa, sin fallar, de estas cosas:

- un "siempre", un "nunca" o un "solo" de un distractor que la correcta no dice;
- una correcta que es la opción más larga en más de 1 de cada 3 preguntas, sin contar las siglas;
- una línea `✓` de menos de 8 palabras;
- dos líneas `✗` del mismo apartado con el mismo matiz, el texto después de los dos puntos;
- una página de un PDF con conceptos que ninguna `fuente` de un concepto cita y que `sin_pregunta` no marca;
- una página que `sin_pregunta` marca y que tiene una pregunta;
- una respuesta del historial con un `id` fuera del banco;
- un `concepto` con menos de 3 preguntas o con más de 3, en un examen con `"congelado": true` en `examen.json`. Un banco congelado es anterior a esa regla, y no se reescribe.

Un aviso de una pista o de una explicación lo juzga el revisor: un absoluto que no delata al distractor se queda.

`examen.json` lleva `sin_pregunta`: para cada PDF, un mapa de la página a la razón de que no tenga pregunta. El revisor juzga cada razón.

```json
"sin_pregunta": {
  "2_Ciclo_de_vida.pdf": { "1": "portada", "2": "índice", "13": "sigue la idea de la página 12" }
}
```

Un tema acaba cuando `banco comprobar` no da ningún aviso de página de su PDF.

## El borrador

El agente creador escribe las preguntas en un borrador en Markdown, y no en el JSON. `banco importar` añade las preguntas del borrador al final de `preguntas.json`:

```bash
cargo run --manifest-path app/Cargo.toml -p banco -- importar examenes/asn ciclo-de-vida borrador.md
```

El borrador tiene un bloque por pregunta:

```
## ciclo-de-vida-defensa-en-profundidad
¿Qué significa aplicar defensa en profundidad?
+ Aplicar medidas de seguridad en todo el ciclo
✓ Cada control puede fallar. Con controles en cada fase, el atacante que pasa uno se encuentra con el siguiente.
- «Un solo control robusto» en el perímetro | una sola barrera es un único punto de fallo
- «Auditar la seguridad una vez» | la seguridad cubre todo el ciclo
```

- El encabezado da el `concepto`. El creador lo escribe antes en `conceptos.json`, con su idea y su `fuente`.
- La línea sin marca es el enunciado, y la línea `+` es la correcta.
- La línea `✓` es obligatoria, y es la primera línea de la `explicacion`.
- Una línea `-` es un distractor, con su clave entre `«»` y su matiz después de un solo `|`. La clave es la parte que separa el distractor de los demás, y no el arranque que comparten. `importar` quita las comillas del distractor y escribe la línea `✗ «clave»: matiz`. Un distractor sin `«»`, o con un `|` que falta o que sobra, da un error.
- Cada pregunta recibe el siguiente `id` libre del apartado: `ciclo-de-vida-127`. Un concepto de sigla recibe la forma `ciclo-de-vida-s15`.
- `importar` solo añade preguntas. Un concepto del borrador que ya está en el banco da un error, y el banco queda igual. Una edición de una pregunta del banco se hace a mano en el JSON.

## Las fuentes

`banco paginas` saca de cada PDF de un examen su texto, con `pdftotext -layout`, y sus imágenes:

- El texto de `<examen>/pdfs/<nombre>.pdf` está en `<examen>/texto/<nombre>.txt`. Un form feed (`\f`) separa las páginas: la página N es el trozo N.
- La imagen de la página N está en `<examen>/paginas/<nombre>-N.webp`. Un diagrama o una tabla se leen en la imagen, porque el texto los pierde.

## El flujo de trabajo

Un agente creador escribe un apartado en su carpeta de trabajo, `<examen>/trabajo/<apartado>/`. La carpeta tiene una copia de `examen.json`, su propio `preguntas.json` y un enlace `pdfs` a los PDFs del examen. Así `banco importar` y `banco comprobar` corren sin tocar el banco del examen.

1. El agente escribe una tanda de unas 10 preguntas en un borrador de su carpeta. Una tanda lleva conceptos enteros, porque `banco importar` no añade preguntas a un concepto del banco.
2. El agente revisa su tanda: compara cada pregunta con su página y con las reglas.
3. El agente corre `banco importar` y `banco comprobar` en su carpeta, y arregla los errores y los avisos. Una pregunta ya importada se arregla a mano en el `preguntas.json` de la carpeta.
4. El agente marca cada página sin pregunta en el `sin_pregunta` de su copia de `examen.json`, con su razón.
5. El agente sigue con la siguiente tanda. El apartado acaba cuando cada página de su PDF tiene un concepto o una razón en `sin_pregunta`, y `banco comprobar` no da ningún aviso de su PDF. Los avisos de los otros PDFs son de otros apartados.

Cada archivo del creador vive en su carpeta de trabajo, también los borradores, porque otros creadores escriben otros apartados a la vez. El creador usa el binario `app/target/debug/banco`, y deja `cargo` al agente principal: varios `cargo` a la vez se bloquean.

Al acabar, el creador devuelve en pocas líneas el número de conceptos y de preguntas, cada aviso que dejó a propósito con su razón, y cada duda sobre una regla.

Los nombres evitan los choques entre apartados:

- Un `concepto` empieza por su apartado: `transporte-mac-hmac`.
- Un concepto de sigla es `sigla-` y la sigla en minúsculas, sin el apartado: `sigla-aead`. Así, una sigla de dos apartados choca en el montaje, y `banco comprobar` la ve.
- Una idea que vive sobre todo en otro PDF es de su apartado. Este apartado la pregunta solo desde lo que su PDF explica.

Cada apartado que escribe un agente lleva una revisión completa al acabar. La revisión del propio creador y el comprobador dejan fallos que solo ve otro agente.

## La revisión completa

Un agente que no escribió las preguntas revisa el apartado entero: compara cada pregunta con su página y con las reglas. El revisor comprueba también que la `fuente` de cada concepto es la página que explica la idea, y juzga cada razón de `sin_pregunta` de su PDF. El revisor acaba cuando juzgó cada pregunta del tema y cada razón de `sin_pregunta` de su PDF. Un fallo señala la página o la regla que rompe la pregunta.

El revisor deja el banco como está, y escribe un informe en JSON con una lista de fallos, en `<examen>/trabajo/revision-<apartado>.json`. Al acabar, devuelve en pocas líneas el número de fallos graves y leves, y los 3 fallos más importantes.

```json
{
  "id": "transporte-007",
  "regla": 3,
  "gravedad": "grave",
  "fallo": "El distractor 2 también es cierto según la página 41.",
  "arreglo": { "distractores": ["…", "…"] }
}
```

- `id` es la pregunta del fallo, o `null` para un fallo sin pregunta, como una página con una idea y sin concepto. El texto del fallo da entonces la página.
- `regla` es el número de la regla rota, y 0 para una `fuente` de concepto equivocada.
- `gravedad` es `grave` para un fallo de la regla 3, y `leve` para el resto, también una `fuente` equivocada.
- `arreglo` da los campos nuevos de la pregunta, o `null` para un fallo sin arreglo claro. Un arreglo cambia solo `pregunta`, `correcta`, `distractores` y `explicacion`. Una `fuente` equivocada se arregla a mano en `conceptos.json`. Su `explicacion` sigue la regla 8.

Un agente aplica los informes:

```bash
cargo run --manifest-path app/Cargo.toml -p banco -- revision examenes/ps examenes/ps/trabajo/revision-*.json
```

`banco revision` lista los fallos sin aplicar con `A mano:`, y ese agente los arregla a mano. Un concepto nuevo entra con un borrador y `banco importar`. Un borrado o un cambio de idea se hacen en el JSON. Después corre el comprobador.

## Un lote de apartados

Un agente principal que escribe varios apartados a la vez, o que rehace un examen entero, sigue [`LOTE.md`](LOTE.md).
