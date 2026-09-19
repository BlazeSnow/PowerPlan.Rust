import { useState } from "react";
import { Toaster } from "sonner";
import { AppSidebar } from "@/components/app-sidebar";
import { HomePage } from "@/pages/home";
import { SettingsPage } from "@/pages/settings";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

export type Page = "home" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("home");

  return (
    <SidebarProvider>
      <AppSidebar page={page} onNavigate={setPage} />
      <SidebarInset className="h-svh">
        <main className="h-full overflow-y-auto p-4">
          {page === "home" ? <HomePage /> : <SettingsPage />}
        </main>
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  );
}
