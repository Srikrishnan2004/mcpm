import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./styles/index.css";
import { App } from "./App";
import { ToastProvider } from "./components/ui/Toast";
import { DataProvider } from "./lib/data";
import { applyTheme } from "./lib/theme";

applyTheme();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ToastProvider>
      <DataProvider>
        <App />
      </DataProvider>
    </ToastProvider>
  </StrictMode>,
);
