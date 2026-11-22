# ASN: Aplicaciones Seguras en la Nube

El examen de la convocatoria Ordinaria I es el viernes 27 de noviembre de 2026, a las 16:00, en la dependencia B5-12. Dura 2 horas como máximo, y tiene 2 partes:

1. Entre 30 y 40 preguntas tipo test, con 3 opciones y una sola correcta.
2. Entre 2 y 3 preguntas de desarrollo o supuestos prácticos.

El test de práctica entrena solo la parte tipo test. El simulacro tiene 35 preguntas y 45 minutos, para dejar tiempo a la parte de desarrollo. El enunciado del examen no dice si un fallo resta, así que `penalizacion` es 0. Cambia `penalizacion` en `examen.json` cuando sepas la regla.

El examen pide un bolígrafo de tinta azul o negra y el NIF, o un documento oficial original de identificación.

## El temario

| `pdf` | El tema | `tema` |
|---|---|---|
| `1_Introduccion.pdf` | Tema 1. Introducción y conceptos básicos | `introduccion` |
| `2_Ciclo_de_vida.pdf` | Tema 2. Seguridad en el ciclo de vida de desarrollo del software | `ciclo-de-vida` |
| `3_Alta_disponibilidad.pdf` | Tema 3. Alta disponibilidad, escalabilidad y tolerancia a fallos | `alta-disponibilidad` |
| `4_Despliegue.pdf` | Tema 4. Configuración y despliegue de aplicaciones en la nube | `despliegue` |
| `5_Monitorizacion.pdf` | Tema 5. Monitorización, detección y respuesta frente a amenazas | `monitorizacion` |
| `6.1_Organizacion.pdf` | Tema 6.1. Organización de la seguridad | `organizacion` |
| `6.2_DFIR.pdf` | Tema 6.2. DFIR | `dfir` |
| `S_Contenedores.pdf` | Seminario de creación y ejecución de contenedores | `contenedores` |

El banco está vacío.

La interfaz está en https://3090.taila51846.ts.net:8601/asn/, en el servidor de `app/`. El servidor sirve todos los exámenes, y la raíz lista cada uno.

El código está en `app/README.md`, y el formato y las reglas del banco están en [`docs/BANCO.md`](../docs/BANCO.md).
