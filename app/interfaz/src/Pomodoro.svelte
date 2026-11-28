<script lang="ts" module>
  // El navegador solo deja sonar tras un clic, así que el audio nace al arrancar.
  // Vive fuera de cada copia del botón, porque cada pantalla monta la suya.
  let audio: AudioContext | undefined;
</script>

<script lang="ts">
  // El cronómetro de pausas: 25 minutos de estudio y 5 de pausa, y 20 de pausa tras cada 3 de estudio.
  // Es un botón de la línea de arriba de cada pantalla. Guarda la hora de fin en `localStorage`,
  // así que sigue al recargar y al cambiar de pantalla. Al acabar una fase, suena y espera un clic.
  type Fase = "estudio" | "pausa" | "larga";
  type Estado = { fase: Fase; hechos: number; fin: number | null; resta: number };
  const MIN = 60_000;
  const duracion: Record<Fase, number> = { estudio: 25 * MIN, pausa: 5 * MIN, larga: 20 * MIN };
  const clave = "pomodoro";

  let e = $state<Estado>(JSON.parse(localStorage.getItem(clave) ?? "null") ?? { fase: "estudio", hechos: 0, fin: null, resta: duracion.estudio });
  let ahora = $state(Date.now());
  const resta = $derived(e.fin ? Math.max(0, e.fin - ahora) : e.resta);
  const segundos = $derived(Math.ceil(resta / 1000));
  // "+5 min" alarga la fase, así que la barra cuenta sobre la fase alargada.
  const hecho = $derived(1 - resta / Math.max(duracion[e.fase], resta));
  const empezada = $derived(!!e.fin || e.resta !== duracion[e.fase]);
  const nombre = $derived(e.fase === "estudio" ? `Estudio · ${(e.hechos % 3) + 1} de 3` : e.fase === "pausa" ? "Pausa" : "Pausa larga");

  const guardar = (n: Estado) => localStorage.setItem(clave, JSON.stringify((e = n)));

  $effect(() => {
    const reloj = setInterval(() => {
      ahora = Date.now();
      if (e.fin && e.fin <= ahora) {
        sonar();
        saltar();
      }
    }, 1000);
    // Otra pestaña cambia el cronómetro, y esta lo sigue.
    const seguir = (ev: StorageEvent) => ev.key === clave && ev.newValue && (e = JSON.parse(ev.newValue));
    addEventListener("storage", seguir);
    return () => {
      clearInterval(reloj);
      removeEventListener("storage", seguir);
    };
  });

  function saltar() {
    const hechos = e.fase === "estudio" ? e.hechos + 1 : e.hechos;
    const fase: Fase = e.fase !== "estudio" ? "estudio" : hechos % 3 ? "pausa" : "larga";
    guardar({ fase, hechos, fin: null, resta: duracion[fase] });
  }

  // Vuelve al primer estudio de la tanda, parado y entero.
  const reiniciar = () => guardar({ fase: "estudio", hechos: 0, fin: null, resta: duracion.estudio });

  function alternar() {
    ahora = Date.now();
    if (e.fin) guardar({ ...e, fin: null, resta: Math.max(0, e.fin - ahora) });
    else {
      audio ??= new AudioContext();
      guardar({ ...e, fin: ahora + e.resta });
    }
  }

  const mas5 = () => guardar(e.fin ? { ...e, fin: e.fin + 5 * MIN } : { ...e, resta: e.resta + 5 * MIN });

  // 3 notas suaves de onda senoidal.
  function sonar() {
    if (!audio) return;
    [660, 880, 660].forEach((f, k) => {
      const t = audio!.currentTime + k * 0.4;
      const o = audio!.createOscillator();
      const g = audio!.createGain();
      o.frequency.value = f;
      g.gain.setValueAtTime(0, t);
      g.gain.linearRampToValueAtTime(0.15, t + 0.05);
      g.gain.exponentialRampToValueAtTime(0.001, t + 0.38);
      o.connect(g).connect(audio!.destination);
      o.start(t);
      o.stop(t + 0.4);
    });
  }
</script>

<!-- El borde y el fondo son los de los otros botones de la línea. Solo el tiempo lleva el color de la fase. -->
<span
  class="pomodoro relative inline-flex h-7 shrink-0 items-center rounded-md border border-linea bg-tarjeta text-[12px] font-semibold"
  class:text-aviso={e.fase === "estudio"}
  class:text-acierto={e.fase !== "estudio"}
>
  <button class="flex h-full cursor-pointer items-center gap-1 pl-2 tabular-nums" title={e.fin ? "Pausar" : empezada ? "Seguir" : "Empezar"} onclick={alternar}>
    {#if e.fin}{@render icono(9, "M8 5v14M16 5v14")}{:else}{@render icono(9, "M6 4l14 8-14 8z", true)}{/if}
    {Math.floor(segundos / 60)}:{String(segundos % 60).padStart(2, "0")}
  </button>
  <button class="peer flex h-full cursor-pointer items-center px-1.5 text-tenue hover:text-texto focus:text-texto" title="Los controles">
    {@render icono(12, "M6 9l6 6 6-6")}
  </button>
  <!-- La barra del avance de la fase, en el borde de abajo. Sin `pointer-events`, porque tapa el borde
       de la flecha: el ratón que baja a los controles la cruza, y el panel se cierra. -->
  <span class="pointer-events-none absolute inset-x-1.5 bottom-0 h-px">
    <span class="block h-full bg-current opacity-60" style:width="{hecho * 100}%"></span>
  </span>
  <!-- El relleno de arriba, y no un margen, deja pasar el ratón de la flecha a los controles. -->
  <span class="absolute top-full right-0 z-30 hidden pt-1 peer-hover:block peer-focus:block hover:block focus-within:block">
    <span class="flex flex-col gap-1 rounded-lg border border-linea bg-tarjeta p-1.5 text-texto shadow-lg">
      <span class="px-1 text-[11px] font-normal whitespace-nowrap text-tenue">{nombre}</span>
      <span class="flex gap-1 [&>button]:flex [&>button]:h-7 [&>button]:flex-1 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:px-2 [&>button]:hover:bg-linea">
        <button title={e.fin ? "Pausar" : empezada ? "Seguir" : "Empezar"} onclick={alternar}>
          {#if e.fin}{@render icono(14, "M8 5v14M16 5v14")}{:else}{@render icono(14, "M6 4l14 8-14 8z", true)}{/if}
        </button>
        <button title="+5 min" onclick={mas5}>+5</button>
        <button title={e.fase === "estudio" ? "Saltar el estudio" : "Saltar la pausa"} onclick={saltar}>{@render icono(14, "M5 4l10 8-10 8zM19 5v14")}</button>
        <button title="Volver al estudio 1" onclick={reiniciar}>{@render icono(14, "M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5")}</button>
      </span>
    </span>
  </span>
</span>

{#snippet icono(lado: number, d: string, relleno = false)}
  <svg width={lado} height={lado} viewBox="0 0 24 24" fill={relleno ? "currentColor" : "none"} stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
    <path {d} />
  </svg>
{/snippet}
