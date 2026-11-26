# El planificador es FSRS, y la probabilidad de acierto es la única medida

Un planificador de reglas fijas da el mismo calendario a una pregunta fácil y a una difícil del mismo concepto. FSRS, el planificador de Anki, aprende del historial la dificultad de cada pregunta y lo que el estudiante la recuerda.

## Las decisiones

- **El crate `fsrs` calcula el estado de cada pregunta desde `respuestas.jsonl`.** El estado es la estabilidad, la dificultad y la hora de la última respuesta. El historial no migra: el estado sale otra vez de las mismas respuestas.
- **La probabilidad de acierto es la única medida.** Una pregunta vence cuando baja de la retención. "Sabes" es una pregunta vista y no vencida, y la sesión ordena las vencidas de la menor probabilidad a la mayor. Un hueco o un paso guardados al lado serían una segunda medida que puede contradecir a la primera.
- **Un fallo y una pregunta en blanco son Again, un acierto con "dudo" es Hard, y un acierto con "seguro" es Good.** Un acierto con "dudo" prueba un recuerdo débil, y no un olvido: Again lo castigaría como un fallo.
- **Una pregunta sin ver acertada con "seguro" es Good, y no Easy.** Easy la manda a unos 8 días, y en ese tiempo un acierto con suerte no vuelve a probarse.
- **Una fallada vence en el acto, aunque FSRS le dé unas horas.** La sesión siguiente la saca primero, hasta que la aciertas. Acertar otra vez una fallada en la misma racha de estudio fija el recuerdo ("successive relearning", Rawson y Dunlosky).
- **La retención va en `examen.json`, con 0.9 si falta.** Subirla al 0.95 pide cerca del doble de repasos. Ese precio sale a cuenta en los días antes de un examen, y no en todo el curso.
- **Los parámetros son los de por defecto de FSRS.** El optimizador necesita cientos de respuestas por examen.

## Las alternativas

- **SM-2, con un paso y un hueco ×2,5.** Da el mismo calendario a todas las preguntas, y un acierto con "dudo" pone el paso a 0 como un fallo.
- **El crate `rs-fsrs`.** Solo planifica, sin optimizador, y va detrás de las versiones de FSRS.
- **El optimizador de FSRS en el navegador.** Entrena una red con `burn`, y el WASM crece.
- **Un acierto con "dudo" como Again.** Mantiene la regla de SM-2, pero tira la información del acierto.
