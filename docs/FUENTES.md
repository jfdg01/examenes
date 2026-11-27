# Las fuentes

## Las páginas de un PDF

`banco paginas <examen>` saca de cada PDF de un examen su texto, con `pdftotext -layout`, y sus imágenes:

- El texto de `<examen>/pdfs/<nombre>.pdf` está en `<examen>/texto/<nombre>.txt`. Un form feed (`\f`) separa las páginas: la página N es el trozo N.
- La imagen de la página N está en `<examen>/paginas/<nombre>-N.webp`. Un diagrama o una tabla se leen en la imagen, porque el texto los pierde.

## La página de una idea

La `fuente` de un concepto es la página que explica la idea, y no una página que solo la nombra. Su número es la página del PDF, y no el número de la diapositiva.

- Una sigla cita la primera página que la explica.
- Una idea marcada en el `README.md` del examen que ningún PDF trae cita la página que más se acerca.
