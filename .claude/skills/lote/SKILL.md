---
name: lote
description: Escribe un banco de un examen por tipo (conceptos, test o desarrollo), con un creador y un revisor por apartado. Úsala con `/lote <examen> <tipo> [apartados]`, o cuando el dueño pide escribir o rehacer los conceptos, las preguntas de test o las de desarrollo de uno o varios apartados.
---

# Un lote

`/lote <examen> <tipo> [apartados]` escribe el banco de un tipo en los apartados dados, o en todos los apartados de los PDFs de `examen.json` sin apartados. `<examen>` es la carpeta, como `examenes/ps`. Tú eres el agente principal: lanzas un creador y un revisor por apartado, en paralelo, y solo tú escribes en el banco del examen.

Un lote corre un solo tipo. El formato, las reglas, el creador y la revisión de cada tipo están en su doc:

- `conceptos`: [`docs/CONCEPTOS.md`](../../../docs/CONCEPTOS.md).
- `test`: [`docs/TEST.md`](../../../docs/TEST.md).
- `desarrollo`: [`docs/DESARROLLO.md`](../../../docs/DESARROLLO.md).

En los comandos, `$B` es `app/target/debug/banco`.

## 1. La preparación

1. Un lote de `test` o de `desarrollo` en un apartado sin conceptos corre antes un lote de `conceptos` en ese apartado, entero, hasta sus arreglos. Después sigue con el lote pedido.
2. Un lote de `test` o de `desarrollo` borra las preguntas de su tipo del apartado. Pregunta al dueño qué pasa con el historial de esos apartados, `respuestas.jsonl` en `test` y `desarrollo.jsonl` en `desarrollo`: `banco importar` vuelve a empezar en el primer `id` libre, y una respuesta vieja señalaría una pregunta nueva. Las opciones son borrar esas respuestas, o guardarlas fuera del repo. `ronda.json` se borra, porque una ronda a medias usa los `id` viejos.
3. Compila `banco` una vez, para el binario de los creadores, y corre `banco preparar`. El comando quita del banco lo que el lote escribe y crea `<examen>/trabajo/<apartado>/` para cada apartado. Falla sin cambiar nada:
   - en `conceptos`, con una pregunta de test o de desarrollo de esos apartados, porque quedaría sin concepto;
   - en `test` y en `desarrollo`, con un apartado sin conceptos, con respuestas de esos apartados en su historial, o con `ronda.json`.

```bash
cargo build --manifest-path app/Cargo.toml -p banco
$B preparar <examen> <tipo> <apartado>...
```

## 2. La creación

Lanza un creador por apartado con esta orden. `<examen>`, `<tipo>`, `<doc>`, `<apartado>`, `<pdf>` y `<N>` cambian en cada orden. Añade lo que el `README.md` del examen marca para ese PDF.

````
Escribe desde cero el banco de <tipo> del apartado `<apartado>` del examen `<examen>`. El PDF es `<pdf>`, de <N> páginas. Escribe en español.

Lee primero `docs/<doc>` entero, `docs/FUENTES.md` y `<examen>/README.md`. Eres el creador de «El creador», y tu carpeta de trabajo es `<examen>/trabajo/<apartado>/`.

```bash
B=app/target/debug/banco
$B importar <examen>/trabajo/<apartado> <tipo> <apartado> <examen>/trabajo/<apartado>/tanda.md
$B comprobar <examen>/trabajo/<apartado>
```
````

Guarda las dudas de cada creador para su revisor.

## 3. El montaje

Cuando acaban todos los creadores, une sus apartados al banco con `banco montar`. Un lote de `conceptos` une también el `sin_pregunta` de sus PDFs. El comando falla sin cambiar nada con un `id` de otro apartado o que ya está en el banco.

```bash
$B montar <examen> <tipo> <apartado>...
```

Una sigla de dos apartados choca en el montaje. Se queda en el apartado cuya página la explica, y sale a mano del `conceptos.json` de la otra carpeta de trabajo. Después:

1. Corre `banco comprobar <examen>`.
2. En `conceptos`, comprueba que cada punto marcado del `README.md` tiene su concepto. En `desarrollo`, que tiene su pregunta. Un punto que ningún creador cubrió entra con un borrador y `banco importar`.
3. Corre los tests, cambia la cifra de preguntas del `README.md` del examen si cambió, y haz el commit. El commit separa el banco de los creadores de los arreglos de la revisión.

## 4. La revisión

Lanza un revisor por apartado con esta orden. Cada revisor recibe las dudas del creador de su apartado.

````
Haz la revisión completa del banco de <tipo> del apartado `<apartado>` del examen `<examen>`. Tú no lo escribiste. Escribe en español.

Lee primero `docs/<doc>`, sobre todo «Las reglas» y «La revisión», `docs/FUENTES.md` y `<examen>/README.md`. Eres el revisor de «La revisión». Tu PDF es `<pdf>`.

Lee solo lo que tiene `"apartado": "<apartado>"` en el banco de `<examen>`, con un filtro de python3: el banco entero satura tu contexto.

Las dudas del creador: <dudas>.
````

## 5. Los arreglos

Cuando acaban todos los revisores, aplica los informes y arregla a mano cada fallo de `A mano:`. Un concepto o una pregunta nuevos entran con un borrador y `banco importar`. Un borrado o un cambio de idea se hacen en el JSON.

```bash
$B revision <examen> <tipo> <examen>/trabajo/revision-*.json
```

El lote acaba cuando cada fallo de `A mano:` tiene su arreglo, `banco comprobar` no da errores ni avisos, y los tests pasan. Entonces haz el commit y borra `<examen>/trabajo/`.
