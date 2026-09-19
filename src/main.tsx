import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "next-themes";
import App from "./App";
import "./i18n";
import "./index.css";

// 屏蔽 WebView2 默认右键菜单
window.addEventListener("contextmenu", (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);
