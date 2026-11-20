# LN

El examen es el 20 de noviembre de 2026, a las 16:30. Dura 25 minutos y tiene 20 preguntas tipo test, con 4 opciones como máximo. Un acierto suma 0,5 puntos, un fallo resta 0,125 y una pregunta en blanco no cuenta.

El banco tiene unas 355 preguntas: unas 245 de RGPD, ISO 27001, ENS y riesgos, y unas 110 del resto. Cada página de los PDFs tiene 2 diapositivas.

La interfaz está en https://3090.taila51846.ts.net:8600/:

```bash
python3 test/serve.py ln
tailscale serve --bg --https=8600 http://127.0.0.1:8601
```

El código, el formato del banco y las reglas de calidad están en `test/README.md`.
