# Las preguntas de desarrollo: `desarrollo.json`

Las preguntas de desarrollo de un examen son una lista en JSON, en `<examen>/desarrollo.json`. Cada una junta de 4 a 8 conceptos de [`CONCEPTOS.md`](CONCEPTOS.md), sus puntos, por [el ADR 0005](adr/0005-el-concepto-es-una-entidad.md). `<examen>` es la carpeta del examen, como `examenes/ps`.

```json
{
  "id": "aplicacion-d003",
  "apartado": "aplicacion",
  "forma": "proceso",
  "enunciado": "Describe el handshake de SSH.",
  "modelo": "El cliente y el servidor intercambian primero sus versiones y sus listas de algoritmos. …",
  "puntos": ["aplicacion-ssh-negociacion", "aplicacion-ssh-clave-de-sesion", "…"]
}
```

- `forma` es una de las 9 formas de «Las formas».
- `modelo` es la respuesta modelo: la que el estudiante memoriza y escribe en el examen.
- `puntos` son los `id` de los conceptos que el modelo cubre, en el orden en que el modelo los cuenta.

## Las formas

| `forma` | Lo que pide | El enunciado tipo |
|---|---|---|
| `objeto` | Qué es algo y para qué sirve | Define X y di para qué sirve. |
| `proceso` | Los pasos de algo, en orden | Describe el handshake de X. |
| `comparacion` | Las diferencias entre 2 cosas | Compara X e Y. |
| `causa-efecto` | Por qué pasa algo, y lo que provoca | Explica por qué el ataque X provoca Y. |
| `estructura` | Las partes de algo y lo que hace cada una | Describe la pila de protocolos de X. |
| `propiedad` | Cómo consigue algo una propiedad de seguridad | Explica cómo da X la integridad y la confidencialidad. |
| `clasificacion` | Los tipos de algo y lo que los separa | Clasifica los X, con un ejemplo de cada tipo. |
| `contramedida` | Lo que defiende de un ataque, y cómo | Explica cómo defiende X del ataque Y. |
| `criterio` | Cómo se elige entre opciones | Di qué X son robustos hoy, y por qué. |

Una pregunta que no encaja en ninguna forma no entra, y va a las dudas del creador.

## La elección de las candidatas

Una candidata es un tema de una pregunta posible: una sección del índice del PDF, o un punto que marcó el profesor en el `README.md` del examen. El creador lista todas las candidatas de su apartado y las ordena así:

1. Lo que marcó el profesor, en el orden del `README.md`.
2. Las secciones del índice del PDF, de las de más páginas a las de menos. Un PDF sin índice usa los títulos de sus diapositivas.

Una candidata entra si junta 4 conceptos o más que ninguna candidata anterior usa. Si no, queda fuera. El corte sale de los conceptos: el apartado acaba cuando ninguna candidata restante junta 4 conceptos libres. Un concepto puede quedar sin pregunta de desarrollo, porque el test lo cubre.

El creador deja la lista en `<examen>/trabajo/<apartado>/candidatas.md`, en su orden, con una línea por candidata:

```
- dentro aplicacion-d003 · marcado: «El handshake de SSH» · proceso · 6 puntos
- fuera · índice: «HPKP», páginas 30 a 31 · 2 conceptos libres
```

## Las reglas

1. **Cada punto es un concepto de su apartado.** Una idea sin concepto va a las dudas del creador, y no entra como texto en el modelo.
2. **Las preguntas no se solapan.** Un concepto es punto de una sola pregunta. Dos preguntas que piden lo mismo con otras palabras son una sola.
3. **Una comparación junta conceptos de las 2 cosas, y es la excepción a la regla 2.** Sus puntos pueden ser puntos de otra pregunta. Entra solo si el temario compara las 2 cosas, con una tabla o una diapositiva de diferencias, o si el profesor la marcó. Su enunciado nombra las 2 cosas, y su modelo da cada diferencia con las 2 partes: "TLS 1.2 hace X, y TLS 1.3 hace Y".
4. **El modelo cubre cada punto.** Quien lee el modelo encuentra la idea de cada punto, con las palabras de la página.
5. **El modelo no dice nada fuera de sus puntos.** Una frase que no cubre ningún punto sale, o su idea entra como punto.
6. **El modelo es una respuesta de examen**, de 4 a 8 frases y de unas 70 a 170 palabras. Es prosa que se memoriza: sigue el orden del enunciado, sin listas ni títulos.
7. **El enunciado pide lo que el modelo responde,** con la forma de su `forma`. No da pistas de los puntos.

## El borrador

El creador escribe las preguntas en un borrador en Markdown, con un bloque por pregunta:

```
## proceso
Describe el handshake de SSH.
> El cliente y el servidor intercambian primero sus versiones y sus listas de algoritmos.
> Después acuerdan la clave de sesión con un Diffie-Hellman que firma la clave del servidor.
- aplicacion-ssh-negociacion
- aplicacion-ssh-clave-de-sesion
```

- El encabezado es la `forma`.
- La línea sin marca es el `enunciado`.
- Cada línea `>` es una parte del `modelo`, y `banco importar` las une con un espacio.
- Cada línea `-` es un punto.

`banco importar` añade las preguntas del borrador al final de `desarrollo.json`, cada una con el siguiente `id` libre de su apartado, como `aplicacion-d004`:

```bash
app/target/debug/banco importar examenes/ps/trabajo/aplicacion desarrollo aplicacion tanda.md
```

Una forma desconocida o un bloque sin enunciado, sin modelo o sin puntos da un error, y el banco queda igual. Una edición de una pregunta del banco se hace a mano en el JSON.

## El comprobador

`banco comprobar <carpeta>` falla con un `id` repetido o que también está en `preguntas.json`, un punto que no es un concepto de su apartado, menos de 4 puntos o más de 8, y un concepto en 2 preguntas que no son de la forma `comparacion`. Avisa, sin fallar, de un modelo de menos de 4 frases o de más de 8.

## El creador

El creador escribe las preguntas de un apartado en su carpeta de trabajo, `<examen>/trabajo/<apartado>/`. La carpeta tiene una copia de `examen.json` y de `conceptos.json`, su propio `desarrollo.json`, un `preguntas.json` vacío y un enlace `pdfs` a los PDFs del examen. Así `banco importar` y `banco comprobar` corren sin tocar el banco del examen.

1. El creador lee los conceptos de su apartado, el índice de su PDF y lo que marcó el profesor.
2. El creador escribe `candidatas.md`, con las candidatas en su orden y el corte.
3. El creador escribe una tanda de unas 5 preguntas en un borrador de su carpeta, en el orden de las candidatas. Lee la página de cada punto, como dice [`FUENTES.md`](FUENTES.md), y compara cada pregunta con las reglas.
4. El creador corre `banco importar` y `banco comprobar` en su carpeta, y arregla los errores y los avisos. Una pregunta ya importada se arregla a mano en el `desarrollo.json` de la carpeta.
5. El creador sigue con la siguiente tanda. El apartado acaba cuando cada candidata dentro tiene su pregunta, y `banco comprobar` no da ningún error ni aviso de desarrollo.

El creador no cambia `conceptos.json`. Una idea sin concepto, o una candidata que no encaja en una forma, va a sus dudas.

Cada archivo del creador vive en su carpeta de trabajo, también los borradores, porque otros creadores escriben otros apartados a la vez. El creador usa el binario `app/target/debug/banco`, y deja `cargo` al agente principal: varios `cargo` a la vez se bloquean.

Al acabar, el creador devuelve en pocas líneas el número de preguntas y de candidatas fuera, cada aviso que dejó a propósito con su razón, y cada duda sobre una regla o un concepto.

## La revisión

Un agente que no escribió las preguntas revisa el apartado entero. El revisor acaba cuando juzgó cada pregunta y el corte de `candidatas.md`:

- **Cada pregunta:** cada punto está en el modelo (regla 4), el modelo no dice nada fuera de sus puntos (regla 5), y el modelo es cierto según la página de cada punto.
- **El corte:** el orden de las candidatas sigue «La elección de las candidatas», y ninguna candidata fuera pesa más que una dentro. Una candidata fuera pesa más si el profesor la marcó, o si su sección tiene más páginas.

El revisor deja el banco como está, y escribe un informe en JSON con una lista de fallos, en `<examen>/trabajo/revision-<apartado>.json`. Al acabar, devuelve en pocas líneas el número de fallos graves y leves, y los 3 fallos más importantes.

```json
{
  "id": "aplicacion-d003",
  "regla": 4,
  "gravedad": "grave",
  "fallo": "El modelo no cubre el punto aplicacion-ssh-clave-de-sesion.",
  "arreglo": { "modelo": "…" }
}
```

- `id` es la pregunta del fallo, o `null` para un fallo del corte. El texto del fallo da entonces la candidata.
- `regla` es el número de la regla rota, y 0 para un fallo del corte.
- `gravedad` es `grave` para un modelo falso según una página, para un punto que el modelo no cubre y para un fallo del corte, y `leve` para el resto.
- `arreglo` da los campos nuevos de la pregunta, o `null` para un fallo sin arreglo claro. Un arreglo cambia solo `forma`, `enunciado`, `modelo` y `puntos`.

`banco revision <examen> desarrollo <informe>...` aplica los arreglos y lista los fallos sin aplicar con `A mano:`.
