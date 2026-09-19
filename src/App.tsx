import { useState } from "react";
import { Toaster } from "sonner";
import { AppSidebar } from "@/components/app-sidebar";
import { TitleBar } from "@/components/title-bar";
import { HomePage } from "@/pages/home";
import { SettingsPage } from "@/pages/settings";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

export type Page = "home" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("home");

  return (
    <SidebarProvider>
      <AppSidebar page={page} onNavigate={setPage} />
      <SidebarInset className="flex h-svh flex-col">
        <TitleBar />
        <main className="flex-1 overflow-y-auto p-4">
          {page === "home" ? <HomePage /> : <SettingsPage />}
        </main>
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  );
}
