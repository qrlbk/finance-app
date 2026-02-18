import React from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import "./index.css";
import { initTheme } from "./stores/themeStore";
import App from "./App";

initTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
