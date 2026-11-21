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
  "explicacion": "SSH acuerda la clave con Diffie-Hellman, y el servidor firma el intercambio. Cifrar la clave con la clave pública del servidor es el intercambio RSA de TLS 1.2, y no el de SSH. Una firma da autenticidad, y no confidencialidad: la clave viajaría a la vista."
}
```

- `id` es la clave del historial, y no cambia nunca. Empieza por el `tema`: `transporte-007`.
- `concepto` agrupa las preguntas de una misma idea.
- `tema` es uno de los `temas` de `examen.json`.
- `fuente` es el PDF y su página de PDF, y no el número de la diapositiva.
- `distractores` tiene `opciones` − 1 opciones. La interfaz baraja la correcta con los distractores.

## El comprobador

```bash
python3 test/comprobar_banco.py ps/preguntas.json
```

`comprobar_banco.py` falla con cada pregunta que rompe el formato: un `id` repetido, un `tema` fuera de la lista, un número de distractores distinto de `opciones` − 1, un distractor igual a la correcta, un `pdf` o una `pagina` que no existen, y un `concepto` con menos de 2 preguntas o con más de 3. También avisa, sin fallar, de las respuestas del historial con un `id` fuera del banco.

Una revisión de agentes que no escribieron las preguntas compara cada pregunta con su página y con las reglas. Los revisores dan la lista de fallos, y un solo agente cambia `preguntas.json`.
