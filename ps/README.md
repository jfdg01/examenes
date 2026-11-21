# PS: Protocolos de Seguridad

El examen es el lunes 23 de noviembre de 2026, a las 16:30. Tiene 2 partes: un test, seguramente de verdadero o falso, y preguntas cortas. El test de práctica entrena las ideas de las 2 partes con 3 opciones por pregunta y una sola correcta. Un fallo resta 1/3 de un acierto, y una pregunta en blanco no cuenta.

El número de preguntas y la duración faltan. El simulacro usa 20 preguntas y 25 minutos, cifras provisionales de `examen.json`.

## El temario

| `pdf` | El tema | `tema` |
|---|---|---|
| `1_Introduccion.pdf` | Tema 1. Introducción | `introduccion` |
| `2_Claves.pdf` | Tema 2. Administración y distribución de claves | `claves` |
| `3_Transporte.pdf` | Tema 3. Capa de transporte | `transporte` |
| `4_Aplicacion.pdf` | Tema 4. Capa de aplicación | `aplicacion` |
| `E_SSL_TLS.pdf` | Seguridad en SSL/TLS: historia de un protocolo, un documento externo | `ssl-tls` |

## Lo que marcó el profesor

Las fechas y los números de RFC no entran en el examen. El resto del temario sí entra. Cada punto de esta lista tiene sus conceptos en el banco:

- Los protocolos más importantes y sus elementos principales, a grandes rasgos.
- La pila de protocolos de SSH, y su Transport Layer Protocol.
- El handshake de SSH, y el handshake de TLS a grandes rasgos.
- Cómo funciona un certificado.
- La integridad y la confidencialidad en S/MIME, como ejemplo de un esquema entero.
- Los algoritmos de cifrado, y cuáles son robustos, con sus tamaños de clave seguros.
- Cómo se firma un mensaje: con qué clave, y sobre qué.
- Cómo se envía la clave compartida: cifrada, firmada, o acordada.
- Por qué un ataque provoca lo que ve la víctima. El caso marcado es el MITM de sslstrip con bettercap: el atacante hace de proxy hasta el sitio real, y el usuario ve una página idéntica en HTTP, o en un dominio parecido como `wwwww-facebook.com`.

## El banco

El banco tiene 662 preguntas: 3 por cada idea de los 5 PDFs, y 1 por cada sigla de seguridad. Una idea que sale en 2 PDFs tiene un solo concepto.

La interfaz está en https://3090.taila51846.ts.net:8606/:

```bash
python3 test/serve.py ps
tailscale serve --bg --https=8606 http://127.0.0.1:8607
```

El código está en `test/README.md`, y el formato y las reglas del banco están en `test/BANCO.md`.
