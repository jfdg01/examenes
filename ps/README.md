# PS: Protocolos de Seguridad

La fecha y el formato del examen faltan. Las cifras de `examen.json` son provisionales: un simulacro de 20 preguntas con 4 opciones, 25 minutos y `penalizacion` 0. Cambia cada cifra en `examen.json` cuando sepas el formato del examen.

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
