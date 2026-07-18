import "@fontsource/dm-sans/latin-400.css";
import "@fontsource/dm-sans/latin-500.css";
import "@fontsource/dm-sans/latin-600.css";
import "@fontsource/dm-sans/latin-700.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { App } from "@/app/App";
import { APP_DISPLAY_NAME } from "@/app/branding";
import "@/integrations/tauri/tauriInventoryBridge";
import "@/app/index.css";

document.title = APP_DISPLAY_NAME;
if (isTauri()) {
  void getCurrentWindow().setTitle(APP_DISPLAY_NAME);
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
