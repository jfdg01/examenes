# PS: Protocolos de Seguridad

El examen es el lunes 30 de noviembre de 2026. Tiene 2 partes: un test, seguramente de verdadero o falso, y preguntas cortas. El test de práctica entrena las ideas de las 2 partes con 3 opciones por pregunta y una sola correcta. Un fallo resta 1/3 de un acierto, y una pregunta en blanco no cuenta.

El número de preguntas y la duración faltan. El simulacro usa 20 preguntas y 25 minutos, cifras provisionales de `examen.json`.

## El temario

| `pdf` | El tema | `apartado` | Lo que trae |
|---|---|---|---|
| `1_Introduccion.pdf` | Tema 1. Introducción | `introduccion` | Las capas, los servicios y los ataques de X.800, el canal seguro, los oráculos, la degradación y PFS |
| `2_Claves.pdf` | Tema 2. Administración y distribución de claves | `claves` | La distribución de claves, el KDC, los certificados X.509, la revocación (CRL y OCSP) y la PKI |
| `3_Transporte.pdf` | Tema 3. Capa de transporte | `transporte` | TLS 1.2, TLS 1.3 y OpenSSL |
| `4_Aplicacion.pdf` | Tema 4. Capa de aplicación | `aplicacion` | La web y HTTPS, HSTS y HPKP, SSH, el correo, DNSSEC, DANE, SPF, DKIM, DMARC y S/MIME |
| `E_SSL_TLS.pdf` | Seguridad en SSL/TLS: historia de un protocolo, un documento externo | `ssl-tls` | Los ataques históricos (sslstrip, BEAST, CRIME, POODLE, Heartbleed…), TLS 1.3 y ECH |

SSH está en el tema 4, y no en el tema 3 de la capa de transporte.

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

El banco tiene 768 preguntas: 3 por cada idea de los 5 PDFs, y 1 por cada sigla de seguridad. Una idea que sale en 2 PDFs tiene un solo concepto.

La interfaz está en https://3090.taila51846.ts.net:8601/ps/, en el servidor de `app/`. El servidor sirve todos los exámenes, y la raíz lista cada uno.

El código está en `app/README.md`, y el formato y las reglas del banco están en [`docs/BANCO.md`](../docs/BANCO.md).
