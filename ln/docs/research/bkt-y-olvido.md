# BKT y olvido para la nota de LN

Cada sección separa **La fuente** (lo que dice, con enlace) de **La conclusión** (mi lectura para LN).

## 1. El BKT clásico y su estimación

**La fuente.**

- BKT tiene cuatro parámetros por skill: L0 (saber al principio), T (aprender en una oportunidad), G (acertar sin saber) y S (fallar sabiendo). Una ayuda pedida cuenta como evidencia de no saber ([Baker, Corbett y Aleven 2008](https://learninganalytics.upenn.edu/ryanbaker/BCA2008W.pdf), que resume a [Corbett y Anderson 1994](https://doi.org/10.1007/BF01099821)).
- Las ecuaciones, tal como las implementa pyBKT ([`predict_onestep.py`](https://github.com/CAHLR/pyBKT/blob/master/source-py/pyBKT/fit/predict_onestep.py), [`EM_fit.py`](https://github.com/CAHLR/pyBKT/blob/master/source-py/pyBKT/fit/EM_fit.py)):
  - La predicción: `P(acierto) = P(L)·(1−S) + (1−P(L))·G`.
  - Tras un acierto: `P(L|a) = P(L)(1−S) / [P(L)(1−S) + (1−P(L))G]`.
  - Tras un fallo: `P(L|f) = P(L)S / [P(L)S + (1−P(L))(1−G)]`.
  - La transición: `P(L') = P(L|obs)·(1−F) + (1−P(L|obs))·T`. En el BKT clásico, F = 0.
- pyBKT ajusta por EM, por skill, con 5 arranques aleatorios por defecto ([`Model.py`](https://github.com/CAHLR/pyBKT/blob/master/source-py/pyBKT/models/Model.py)). Sortea G en [0, 0.4), S en [0, 0.3) y T en [0, 0.4) ([`random_model_uni.py`](https://github.com/CAHLR/pyBKT/blob/master/source-py/pyBKT/generate/random_model_uni.py)). El M-step divide las softcounts esperadas ([`M_step.py`](https://github.com/CAHLR/pyBKT/blob/master/source-py/pyBKT/fit/M_step.py)). Una observación vacía deja la likelihood en 1, así que no actualiza.
- La identifiabilidad: datos iguales encajan con parámetros distintos ([Beck y Chang 2007](https://doi.org/10.1007/978-3-540-73078-1_17), según Baker 2008). [van de Sande 2013](https://jedm.educationaldatamining.org/index.php/JEDM/article/download/35/pdf_27) lo explica: la curva media depende solo de S, T y `A = (1−S−G)(1−L0)`, así que G y L0 se compensan.
- La degeneración: un modelo es degenerado en teoría si G > 0.5 o S > 0.5. Los tutores acotan G a [0, 0.3] y S a [0, 0.1]. Las priors de Dirichlet de Beck dan un 23 % de skills que fallan una prueba empírica de degeneración (Baker 2008). van de Sande exige `G + S < 1` y `T < 1 − S/(1−G)`.
- El ajuste por fuerza bruta busca en una rejilla de dos decimales ([Ritter et al. 2009](https://www.educationaldatamining.org/EDM2009/uploads/proceedings/ritter.pdf); el [código de Baker](https://ryanbaker.c3l.ai/ryanbaker/edmtools.html)). Ritter reduce 2400 skills a 23 grupos de parámetros sin perder comportamiento. Con pocos alumnos, los grupos predicen mejor que el ajuste por skill: el ajuste por skill sobreajusta.

**La conclusión.** LN tiene unas 1.7 respuestas por concepto. Un ajuste por concepto no se puede hacer. Unos parámetros globales para los 165 conceptos son el caso extremo de los grupos de Ritter, y es el único que los datos sostienen. La prior de Beck entra como pseudo-counts en el M-step. Las cotas de Baker y de van de Sande impiden la degeneración.

## 2. El olvido y los modelos de memoria

**La fuente.**

- BKT+F añade F = P(olvidar). Entre dos pruebas de un skill, F se aplica una vez por cada prueba intermedia: `1 − (1−F)^n`. En Assistments, el AUC sube de 0.73 a 0.83 ([Khajah, Lindsey y Mozer 2016](https://arxiv.org/abs/1604.02416)).
- KT-Forget aplica el olvido solo en una respuesta de un día nuevo. El error de KT en un día nuevo es mucho mayor que en el mismo día. KT-Forget mejora la predicción de forma significativa ([Qiu et al. 2011](https://educationaldatamining.org/EDM2011/wp-content/uploads/proc/edm11_proceedings.pdf), pp. 139-148).
- La half-life regression (HLR) predice `p = 2^(−Δ/h)` y `h = 2^(Θ·x)`, donde x cuenta los aciertos y los fallos. Leitner es HLR con pesos fijos: `Θ = {aciertos: +1, fallos: −1}` ([Settles y Meeder 2016](https://aclanthology.org/P16-1174.pdf), apéndice). HLR necesita millones de datos para sus pesos.
- DASH es `logistic(a_alumno − d_item + Σ_w φ_w·log(1+c_w) − ψ_w·log(1+n_w))`, con conteos por ventanas de tiempo ([Lindsey et al. 2014](https://doi.org/10.1177/0956797613504302)). Su fuerza viene de los datos de una población de alumnos.
- En ACT-R, cada práctica deja una huella que decae como potencia del tiempo. El decaimiento de una práctica sube con la activación del momento ([Pavlik y Anderson 2003](https://www.gwern.net/docs/www/act-r.psy.cmu.edu/f4b85d71f1bd93309c094119607097a071bb09e3.pdf); la versión de revista es [Pavlik y Anderson 2005](https://doi.org/10.1207/s15516709cog0000_14)).
- En FSRS, la retrievability es `R = (1 + FACTOR·t/S)^(−w20)`, con R(S) = 0.9. El modelo tiene 21 parámetros. Con menos de 512 repasos de días distintos, el optimizador devuelve los parámetros por defecto ([`scheduler.py`](https://github.com/open-spaced-repetition/py-fsrs/blob/main/fsrs/scheduler.py), [`optimizer.py`](https://github.com/open-spaced-repetition/py-fsrs/blob/main/fsrs/optimizer.py)).

**La conclusión.** HLR, DASH, ACT-R y FSRS modelan el recuerdo sin guess. Con 4 opciones, combinarlos con BKT es así: la retención da P(L), y G y S dan el acierto. Con unas 280 respuestas en 15 días, FSRS usaría sus valores de Anki, y HLR y DASH no tienen datos para sus pesos. El modelo más simple que funciona es KT-Forget: un F global por día de calendario, `P(L) ← P(L)·(1−F)^días`. Un solo parámetro.

## 3. Varias preguntas por concepto

**La fuente.** KT-IDEM da a cada pregunta su G y su S. En los datos de Cognitive Tutor, mejora a KT solo en los conjuntos con más de 6 datos por pregunta. El artículo atribuye el resto al exceso de parámetros ([Pardos y Heffernan 2011](https://web.cs.wpi.edu/~nth/pubs_and_grants/papers/2011/UMAP/Pardos%20Introducing%20Item%20Difficulty.pdf)). pyBKT implementa la idea con `multigs`: G y S por clase de pregunta ([README de pyBKT](https://github.com/CAHLR/pyBKT)).

**La conclusión.** LN tiene menos de 1 dato por pregunta, así que la dificultad por pregunta queda fuera. Dos clases sí caben: "pregunta nueva para el concepto" y "pregunta repetida". Así la memoria de una pregunta concreta entra en un G propio y no ensucia el G de las hermanas. La regla ya decidida (una prueba es una pregunta distinta de la anterior) es la versión más simple, y basta para empezar.

## 4. El seguro/dudo y el blanco

**La fuente.**

- No encontré un trabajo que meta la confianza declarada en BKT.
- Lo más cercano es FSRS. Sus notas Again, Hard, Good y Easy son una autoevaluación: un Hard es un acierto cuyo aumento de stability se multiplica por un `hard_penalty` (`scheduler.py`).
- KT con crédito parcial cambia el nodo binario de la respuesta por uno continuo, y mejora la predicción ([Wang y Heffernan 2013](https://doi.org/10.1007/978-3-642-39112-5_19); leí solo el resumen).
- Para el blanco hay dos convenciones. En pyBKT, una respuesta vacía no actualiza (`EM_fit.py`). En los Cognitive Tutors, pedir ayuda cuenta como no saber (Baker 2008).

**La conclusión.** El seguro/dudo cabe en BKT como una emisión de cuatro salidas por estado: acierto-seguro, acierto-dudo, fallo-dudo y fallo-seguro. Son 6 parámetros globales de emisión, y el historial tiene 49 o más casos en tres de las salidas. Un blanco es una renuncia, igual que pedir ayuda, y cuenta como fallo-dudo. Así coincide con el [ADR del blanco](../adr/0001-el-blanco-lleva-su-propia-clave.md).

## 5. De las probabilidades a la nota, y la calibración

**La fuente.**

- El número de aciertos de 20 preguntas con p distintas sigue una Poisson-binomial ([Hong 2013](https://doi.org/10.1016/j.csda.2012.10.006)).
- Un modelo está calibrado si, entre sus predicciones cercanas a 0.8, acierta un 80 %. La curva de calibración agrupa las predicciones en bins. El Brier score y el log loss mezclan calibración y discriminación, así que un Brier menor no prueba una calibración mejor ([scikit-learn](https://scikit-learn.org/stable/modules/calibration.html), que cita la descomposición de Murphy 1973).

**La conclusión.**

- Con A aciertos y 20 contestadas, los puntos son `A − (20−A)/4 = 1.25·A − 5`, y la nota es la mitad.
- Contestar vale la pena si `p − (1−p)/4 > 0`, es decir, si p > 0.2. Con G ≥ 0.25, el modelo nunca recomienda un blanco. El blanco sale solo de lo que el estudiante sabe en el examen.
- Una simulación de Monte Carlo cubre las tres fuentes de incertidumbre. Sortea 20 conceptos, sortea unos parámetros de su posterior o de un bootstrap, y sortea los aciertos.

## Las conclusiones para LN

**El modelo.** BKT por concepto con parámetros globales y una emisión de cuatro salidas. Para un concepto c con `P(L_c)`:

```
salidas o ∈ {AS, AD, FD, FS}     (acierto/fallo × seguro/dudo; blanco = FD)
e1(o) = P(o | sabe),  e0(o) = P(o | no sabe)
S = e1(FD) + e1(FS)               G = e0(AS) + e0(AD)

actualización:  P(L|o) = P(L)·e1(o) / [P(L)·e1(o) + (1−P(L))·e0(o)]
transición:     P(L') = P(L|o) + (1 − P(L|o))·T
olvido (fase 2): P(L) ← P(L)·(1−F)^días
predicción:     P(acierto_c) = P(L_c)·(1−S) + (1−P(L_c))·G
```

Solo actualiza una prueba: una respuesta a una pregunta distinta de la anterior del concepto.

**El axioma.** Con `P(L_c) = 1` para todo c, la nota esperada es `10·(1 − 1.25·S)`, y `P(10) = (1−S)^20`. Con S = 0.02, la nota esperada es 9.75. El 73 % de hoy no es un techo. Tras un solo acierto-seguro, `P(L)` queda lejos de 1, porque G es alto con 4 opciones.

**El ajuste.** Hay 8 parámetros globales: L0, T y las 6 libres de e0 y e1. El ajuste usa EM (Baum-Welch con dos estados), unas 60 líneas de JS, con varios arranques. El M-step suma pseudo-counts como prior:

- e1 empieza en (0.90, 0.07, 0.02, 0.01), así que S empieza en 0.03.
- e0 empieza con G cerca de 0.3.
- Las cotas son G < 0.5, S < 0.5 y `G + S < 1`. Si el ajuste choca con una cota, lo dice.

Un ajuste por concepto queda fuera, por falta de datos.

**El olvido.** Hoy, F = 0. Antes de añadirlo, la calibración se mira aparte en las respuestas de un día nuevo y en las del mismo día, como hace Qiu. Si el modelo sobreestima las de un día nuevo, entra un F global por día. FSRS y HLR esperan a tener más de 512 repasos.

**La nota.** La nota sale de una simulación de Monte Carlo con 10 000 exámenes: 20 conceptos al azar, unos parámetros de un bootstrap del historial, y los aciertos. La tarjeta muestra la media, un intervalo del 80 % y `P(nota ≥ x)`.

**La calibración.**

- La predicción de cada prueba se hace antes de su actualización, como en `predict_onestep`.
- El ajuste usa el primer 80 % del historial por fecha, y la prueba usa el último 20 %.
- La medida es el Brier y el log loss frente a una tasa constante, más una curva de 5 bins por cuantiles con un intervalo binomial por bin. Con unas 280 respuestas, más bins quedan vacíos o con ruido.
- La comprobación del axioma es aparte. Los conceptos con `P(L) > 0.9` deben acertar sus hermanas por encima del 90 %.

## Lo que no encontré

- Un trabajo que meta la confianza declarada (seguro/dudo) en BKT o en un modelo de memoria.
- Un trabajo que mida, en knowledge tracing, el efecto de repetir la misma pregunta frente a una hermana.
- El texto completo de Corbett y Anderson 1994 y de Beck y Chang 2007, que están tras un muro de pago. Sus afirmaciones vienen de Baker 2008, de van de Sande 2013 y del código de pyBKT.
- El texto de Pavlik y Anderson 2005. Usé su artículo de ICCM 2003, que tiene el mismo modelo de activación.
- Un número mínimo de respuestas para BKT con priors. El 512 es una regla del optimizador de FSRS, no de BKT.

## Las decisiones de LN

- La unidad es el concepto. El examen pregunta los mismos conceptos del banco con otras preguntas.
- Una respuesta tiene 3 resultados: "lo sé y acierto" (acierto con seguro), "creo que lo sé y fallo" (fallo con seguro) y "no lo sé" (dudo o blanco). Un acierto con dudo es una adivinanza, y no sube la probabilidad de saber el concepto. La tasa de acierto de una duda contestada da solo el consejo del examen.
- El lapsus sale de los datos.
- "Lo sabes" es una probabilidad de saber el concepto del 90 % o más.
- La sesión elige los 12 conceptos con la probabilidad más baja de saberlos. Los pesos de un sorteo quedan fuera, por la simulación de abajo.
- La nota sale de una simulación con un bootstrap de los parámetros.

## El ajuste y la simulación con el historial del 19 de noviembre de 2026

[`bkt/bkt.mjs`](bkt/bkt.mjs) ajusta el BKT de 3 resultados con EM, sobre 285 respuestas. Da L0 = 0,18 y T = 0,29. Un concepto sabido da "lo sé y acierto" un 94 %, "creo que lo sé y fallo" un 1 % y "no lo sé" un 5 %. Un concepto no sabido da "lo sé y acierto" un 28 %. Un concepto sin ver llega al 90 % con 3 aciertos con seguro seguidos, en preguntas hermanas.

[`bkt/sim.mjs`](bkt/sim.mjs) compara 7 formas de elegir las 12 preguntas, con 400 estudiantes simulados desde el estado de ese día. Con 8 sesiones:

| La forma de elegir | P(≥ 5) | P(≥ 7) | Conceptos sabidos |
|---|---|---|---|
| Los 12 con la probabilidad más baja | 98,0 % | 54,5 % | 87,0 |
| Caja 1, sin ver, sorteo por caja (el modelo de `main`) | 97,9 % | 53,8 % | 86,3 |
| Sorteo con un peso de (1 − P)⁴, mínimo 0,02 | 97,9 % | 53,7 % | 86,2 |
| Fallados primero, peso de 1 − P, mínimo 0,1 | 97,7 % | 51,8 % | 84,3 |
| Al azar | 97,2 % | 49,2 % | 81,7 |

La ganancia de una prueba es T · (1 − P), y es más alta en el concepto menos sabido. Las diferencias entre las formas son de 1 a 5 puntos en el 7. El número de sesiones pesa más. La simulación usa el mismo modelo para elegir y para juzgar, y no tiene olvido. Con un olvido, un concepto sabido vuelve a necesitar repasos, y un peso mínimo vuelve a importar.

La app sigue con el modelo de `main` hasta después del examen. El BKT no está implementado.
