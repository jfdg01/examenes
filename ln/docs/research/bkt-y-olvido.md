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

