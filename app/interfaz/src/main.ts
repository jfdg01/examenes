import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import Lista from "./Lista.svelte";
import Plan from "./Plan.svelte";
import cargarNucleo from "./nucleo/nucleo.js";

// La raíz muestra la lista de los exámenes, `/?plan` el plan de estudio, y `/<carpeta>/` su examen.
const pantalla = location.pathname !== "/" ? App : location.search === "?plan" ? Plan : Lista;
cargarNucleo().then(() => mount(pantalla, { target: document.getElementById("app")! }));
