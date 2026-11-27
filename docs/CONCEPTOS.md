# Los conceptos: `conceptos.json`

Los conceptos de un examen son una lista en JSON, en `<examen>/conceptos.json`. Las preguntas de test y las de desarrollo salen de ellos, por [el ADR 0005](adr/0005-el-concepto-es-una-entidad.md). `<examen>` es la carpeta del examen, como `examenes/ps`.

```json
{
  "id": "transporte-ssh-clave-de-sesion",
  "apartado": "transporte",
  "idea": "En SSH, el cliente y el servidor acuerdan la clave de sesión con un Diffie-Hellman que firma la clave del servidor.",
  "fuente": { "pdf": "3_Transporte.pdf", "pagina": 41 }
}
```

- `id` empieza por su apartado: `transporte-mac-hmac`. Un concepto de sigla es `sigla-` y la sigla en minúsculas, sin el apartado: `sigla-aead`. Así, una sigla de dos apartados choca en el montaje.
- `apartado` es un apartado del PDF de su `fuente`, en `examen.json`.
- `idea` es la idea en una frase que se entiende sola. La solución de una pregunta de desarrollo la enseña como un punto.
- `fuente` es el PDF y su página de PDF, y no el número de la diapositiva. [`FUENTES.md`](FUENTES.md) dice qué página es.

## Las reglas

Valen para cada banco, salvo los congelados, como ASN y CA, que no se reescriben.

1. **Un concepto es una idea del temario.** Una idea es una regla, un mecanismo, la diferencia entre 2 cosas o lo que hace algo. Una página da tantos conceptos como ideas tiene, de 0 a varios. Una fecha, un número de RFC o de norma, o el nombre de quien publica algo no son una idea.
2. **La idea se entiende sola.** Da su contexto: el protocolo, el mecanismo o el proceso del que habla. Quien no ha visto la diapositiva entiende la frase. La idea no remite al documento: "según el PDF" o "en la diapositiva" no van.
3. **Un dato que el temario da de 2 formas no tiene concepto.** Una pregunta de test sobre él no tiene una sola respuesta defendible.
4. **Una sigla de seguridad es un concepto propio.** Su idea dice el significado de la sigla y lo que nombra. Las siglas generales de informática, como TCP, IP o ASCII, quedan fuera.
5. **Lo que el profesor marcó tiene su concepto.** El `README.md` de cada examen dice lo que el profesor marcó como importante. Una idea marcada que ningún PDF trae es cierta según el `README.md`.
6. **Una idea que vive sobre todo en otro PDF es del apartado de ese PDF.** Este apartado la recoge solo en lo que su PDF explica.

Un concepto que pasa a ser otra idea lleva un `id` nuevo. Un lote de conceptos corre solo en un apartado sin preguntas, porque una pregunta nombra su concepto por el `id`.

## El borrador

El creador escribe los conceptos en un borrador en Markdown, con un bloque por concepto:

```
## transporte-ssh-clave-de-sesion
En SSH, el cliente y el servidor acuerdan la clave de sesión con un Diffie-Hellman que firma la clave del servidor.
@ 41

## sigla-aead
AEAD (Authenticated Encryption with Associated Data) es un cifrado que da a la vez confidencialidad e integridad.
@ 12
```

- El encabezado es el `id`, y empieza por el apartado o por `sigla-`.
- La línea sin marca es la `idea`, y la línea `@` la página de la `fuente`. El PDF es el del apartado.

`banco importar` añade los conceptos del borrador al final de `conceptos.json`:

```bash
app/target/debug/banco importar examenes/ps/trabajo/transporte conceptos transporte tanda.md
```

Un `id` que ya está en el banco da un error, y el banco queda igual. Una edición de un concepto del banco se hace a mano en el JSON.

## El comprobador

`banco comprobar <carpeta>` falla sin `conceptos.json`, salvo en un examen congelado. En los conceptos, falla con un `id` repetido, un `apartado` que no es de su PDF, y un `pdf` o una `pagina` que no existen. También avisa, sin fallar, de estas cosas:

- una página de un PDF con conceptos que ninguna `fuente` cita y que `sin_pregunta` no marca;
- una página que `sin_pregunta` marca y que es la `fuente` de un concepto.

`examen.json` lleva `sin_pregunta`: para cada PDF, un mapa de la página a la razón de que no tenga concepto. El revisor juzga cada razón.

```json
"sin_pregunta": {
  "2_Ciclo_de_vida.pdf": { "1": "portada", "2": "índice", "13": "sigue la idea de la página 12" }
}
```

Un tema acaba cuando `banco comprobar` no da ningún aviso de página de su PDF.

## El creador

El creador escribe un apartado en su carpeta de trabajo, `<examen>/trabajo/<apartado>/`. La carpeta tiene una copia de `examen.json`, su propio `conceptos.json`, un `preguntas.json` vacío y un enlace `pdfs` a los PDFs del examen. Así `banco importar` y `banco comprobar` corren sin tocar el banco del examen.

1. El creador lee cada página de su PDF, el texto y la imagen, como dice [`FUENTES.md`](FUENTES.md).
2. El creador escribe una tanda de unos 15 conceptos en un borrador de su carpeta, y la compara con sus páginas y con las reglas.
3. El creador corre `banco importar` y `banco comprobar` en su carpeta, y arregla los errores y los avisos. Un concepto ya importado se arregla a mano en el `conceptos.json` de la carpeta.
4. El creador marca cada página sin concepto en el `sin_pregunta` de su copia de `examen.json`, con su razón.
5. El creador sigue con la siguiente tanda. El apartado acaba cuando cada página de su PDF tiene un concepto o una razón en `sin_pregunta`, y `banco comprobar` no da ningún aviso de su PDF. Los avisos de los otros PDFs son de otros apartados.

Cada archivo del creador vive en su carpeta de trabajo, también los borradores, porque otros creadores escriben otros apartados a la vez. El creador usa el binario `app/target/debug/banco`, y deja `cargo` al agente principal: varios `cargo` a la vez se bloquean.

Al acabar, el creador devuelve en pocas líneas el número de conceptos, cada aviso que dejó a propósito con su razón, y cada duda sobre una regla.

## La revisión

Un agente que no escribió los conceptos revisa el apartado entero: compara cada concepto con su página y con las reglas. El revisor comprueba que la `fuente` de cada concepto es la página que explica la idea, que ninguna idea de una página falta, y juzga cada razón de `sin_pregunta` de su PDF. El revisor acaba cuando juzgó cada concepto y cada razón. Un fallo señala la página o la regla que rompe el concepto.

El revisor deja el banco como está, y escribe un informe en JSON con una lista de fallos, en `<examen>/trabajo/revision-<apartado>.json`. Al acabar, devuelve en pocas líneas el número de fallos graves y leves, y los 3 fallos más importantes.

```json
{
  "id": "transporte-ssh-clave-de-sesion",
  "regla": 2,
  "gravedad": "leve",
  "fallo": "La idea no dice que habla de SSH.",
  "arreglo": { "idea": "…" }
}
```

- `id` es el concepto del fallo, o `null` para un fallo sin concepto, como una página con una idea y sin concepto. El texto del fallo da entonces la página.
- `regla` es el número de la regla rota, y 0 para una `fuente` equivocada.
- `gravedad` es `grave` para una idea falsa según su página o para una idea que falta, y `leve` para el resto.
- `arreglo` da los campos nuevos del concepto, o `null` para un fallo sin arreglo claro. Un arreglo cambia solo `idea` y `fuente`.

`banco revision <examen> conceptos <informe>...` aplica los arreglos y lista los fallos sin aplicar con `A mano:`.
