import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Toaster, toast } from "sonner";
import { useTranslation } from "react-i18next";
import { useSystemTheme } from "@/lib/use-system-theme";
import { AppSidebar } from "@/components/app-sidebar";
import { HomePage } from "@/pages/home";
import { SettingsPage } from "@/pages/settings";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

export type Page = "home" | "settings";

export default function App() {
  const { t } = useTranslation();
  const [page, setPage] = useState<Page>("home");
  const systemTheme = useSystemTheme();

  useEffect(() => {
    // 托盘操作失败（如系统禁用自启动后尝试开启）：后端打开主窗口并
    // 发送文案键，经软件内自绘 toast 提示（不受系统通知设置影响）
    const unlistenError = listen<{
      key: string;
      args?: Record<string, string>;
    }>("autostart-error", (e) => {
      toast.error(t(e.payload.key, e.payload.args));
    });
    return () => {
      void unlistenError.then((fn) => fn());
    };
  }, [t]);

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
