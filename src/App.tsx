import { useState } from "react";
import { Toaster } from "sonner";
import { AppSidebar } from "@/components/app-sidebar";
import { HomePage } from "@/pages/home";
import { SettingsPage } from "@/pages/settings";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

export type Page = "home" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("home");

  return (
    <SidebarProvider>
      <AppSidebar page={page} onNavigate={setPage} />
      {/* 标题栏已交还系统，仅内容区顶部保留侧边栏伸缩按钮 */}
      <SidebarInset className="flex h-svh flex-col">
        <div className="flex h-10 shrink-0 items-center gap-1 border-b px-2">
          <SidebarTrigger />
        </div>
        <main className="flex-1 overflow-y-auto p-4">
          {page === "home" ? <HomePage /> : <SettingsPage />}
        </main>
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  );
}
