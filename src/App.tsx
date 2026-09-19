import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Toaster } from "sonner";
import { applySystemTheme } from "@/lib/theme";
import { AppSidebar } from "@/components/app-sidebar";
import { HomePage } from "@/pages/home";
import { SettingsPage } from "@/pages/settings";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

export type Page = "home" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [systemTheme, setSystemTheme] = useState<"light" | "dark">("light");

  useEffect(() => {
    // 系统深浅主题由后端桥接（WebView2 的 prefers-color-scheme 不保证实时更新）：
    // 事件实时推送，命令挂载时兜底（延迟一拍，避免被 next-themes 挂载效果覆盖）；
    // 同一状态驱动文档 dark 类与 Toast 主题
    const unlistenPromise = listen<"light" | "dark">("system-theme", (e) => {
      setSystemTheme(e.payload);
      applySystemTheme(e.payload);
    });
    const timer = setTimeout(() => {
      void invoke<"light" | "dark">("system_theme").then((theme) => {
        setSystemTheme(theme);
        applySystemTheme(theme);
      });
    }, 50);
    return () => {
      clearTimeout(timer);
      void unlistenPromise.then((fn) => fn());
    };
  }, []);

  return (
    <SidebarProvider>
      <AppSidebar page={page} onNavigate={setPage} />
      <SidebarInset className="h-svh">
        <main className="h-full overflow-y-auto p-4">
          {page === "home" ? <HomePage /> : <SettingsPage />}
        </main>
      </SidebarInset>
      <Toaster theme={systemTheme} />
    </SidebarProvider>
  );
}
