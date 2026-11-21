# PS: Protocolos de Seguridad

El examen es el lunes 23 de noviembre de 2026, a las 16:30. Tiene preguntas tipo test, con 3 opciones y una sola correcta. Un fallo resta 1/3 de un acierto, y una pregunta en blanco no cuenta.

El número de preguntas y la duración faltan. El simulacro usa 20 preguntas y 25 minutos, cifras provisionales de `examen.json`.

## El temario

| `pdf` | El tema | `tema` |
|---|---|---|
| `1_Introduccion.pdf` | Tema 1. Introducción | `introduccion` |
| `2_Claves.pdf` | Tema 2. Administración y distribución de claves | `claves` |
| `3_Transporte.pdf` | Tema 3. Capa de transporte | `transporte` |
| `4_Aplicacion.pdf` | Tema 4. Capa de aplicación | `aplicacion` |
| `E_SSL_TLS.pdf` | Seguridad en SSL/TLS: historia de un protocolo, un documento externo | `ssl-tls` |

El banco está vacío.

La interfaz está en https://3090.taila51846.ts.net:8606/:

```bash
python3 test/serve.py ps
tailscale serve --bg --https=8606 http://127.0.0.1:8607
```

El código, el formato del banco y las reglas de calidad están en `test/README.md`.
