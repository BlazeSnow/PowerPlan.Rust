import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { ThemeProvider } from "next-themes";
import App from "./App";
import { applySystemTheme } from "./lib/theme";
import "./i18n";
import "./index.css";

// 屏蔽 WebView2 默认右键菜单
window.addEventListener("contextmenu", (e) => e.preventDefault());

// 跟随系统深浅主题：WebView2 的 prefers-color-scheme 不保证随系统实时
// 更新，由后端轮询注册表并 emit system-theme 事件桥接。挂载后延迟一拍
// 应用初始值，避免被 next-themes 的挂载效果覆盖；此后仅事件驱动切换。
function SystemThemeSync() {
  useEffect(() => {
    const unlistenPromise = listen<"light" | "dark">("system-theme", (e) =>
      applySystemTheme(e.payload),
    );
    const timer = setTimeout(() => {
      void invoke<"light" | "dark">("system_theme").then(applySystemTheme);
    }, 50);
    return () => {
      clearTimeout(timer);
      void unlistenPromise.then((fn) => fn());
    };
  }, []);
  return null;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <SystemThemeSync />
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);
