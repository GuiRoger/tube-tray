import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { isMac } from "./lib/platform";
import "./styles/app.css";

document.documentElement.classList.add(isMac ? "mac" : "win");
// menu de contexto do navegador não faz sentido num app de bandeja
document.addEventListener("contextmenu", (e) => {
  if (!(e.target instanceof HTMLInputElement)) e.preventDefault();
});

if (import.meta.env.VITE_MOCK) await import("./dev/mock");

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
