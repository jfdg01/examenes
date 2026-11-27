# El concepto es una entidad, y las preguntas salen de él

Una pregunta de desarrollo junta varias ideas del temario, y una pregunta de test pone a prueba una. Las dos necesitan la misma lista de ideas, con su página. Un lote extrae primero los conceptos de un apartado, y los lotes de test y de desarrollo escriben sus preguntas desde ellos.

## Las decisiones

- **`conceptos.json` guarda cada concepto con su `id`, su `apartado`, su `idea` y su `fuente`.** La `idea` es una frase que se entiende sola. Una pregunta de test nombra su `concepto` y toma de él la página. La página vive en un solo sitio.
- **Cada punto de una pregunta de desarrollo es un concepto.** La solución enseña la `idea` de cada punto, con el enlace a su página.
- **El tamaño del banco de desarrollo sale de los conceptos.** Una pregunta de desarrollo junta de 4 a 8 conceptos, y un concepto es punto de una sola pregunta, salvo en una comparación. Una cifra atada al formato del examen da un banco enorme con 8 preguntas de desarrollo, y uno corto con 3.
- **Un lote de conceptos corre solo en un apartado sin preguntas.** Rehacer los conceptos de un apartado con preguntas deja preguntas que nombran conceptos que ya no existen.

## Las alternativas

- **El concepto como un slug dentro de cada pregunta de test.** El desarrollo no tiene de dónde sacar sus puntos, y cada pregunta repite la página.
- **Un punto de desarrollo con su propio texto y su página, sin concepto.** Cubre una idea que ningún concepto recoge, pero el banco tiene dos listas de ideas que se pueden contradecir. Si una pregunta de desarrollo no se puede escribir solo con conceptos, esta alternativa vuelve.
- **Un presupuesto de 8 preguntas de desarrollo por cada una del examen.** Depende de una cifra del examen que cambia de un curso a otro.
