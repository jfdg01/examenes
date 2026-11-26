import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import Lista from "./Lista.svelte";
import cargarNucleo from "./nucleo/nucleo.js";

// La raíz muestra la lista de los exámenes, y `/<carpeta>/` su examen.
const pantalla = location.pathname !== "/" ? App : Lista;
cargarNucleo().then(() => mount(pantalla, { target: document.getElementById("app")! }));
