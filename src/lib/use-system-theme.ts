import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { applySystemTheme } from "@/lib/theme";

/** 系统深浅主题状态：事件实时推送 + 命令挂载时兜底（延迟一拍，避免被
 * next-themes 挂载效果覆盖）；同一状态驱动文档 dark 类与 Toast 主题。 */
export function useSystemTheme(): "light" | "dark" {
  const [theme, setTheme] = useState<"light" | "dark">("light");

  useEffect(() => {
    // WebView2 的 prefers-color-scheme 不保证随系统实时更新，由后端桥接
    const unlistenPromise = listen<"light" | "dark">("system-theme", (e) => {
      setTheme(e.payload);
      applySystemTheme(e.payload);
    });
    const timer = setTimeout(() => {
      void invoke<"light" | "dark">("system_theme").then((value) => {
        setTheme(value);
        applySystemTheme(value);
      });
    }, 50);
    return () => {
      clearTimeout(timer);
      void unlistenPromise.then((fn) => fn());
    };
  }, []);

  return theme;
}
