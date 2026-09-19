import { useTranslation } from "react-i18next";
import { House, Settings2 } from "lucide-react";
import type { Page } from "@/App";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

const NAV_ITEMS: { page: Page; labelKey: string; icon: typeof House }[] = [
  { page: "home", labelKey: "Shell.Home", icon: House },
  { page: "settings", labelKey: "Shell.Settings", icon: Settings2 },
];

/** 侧边栏：切换主页与设置页，伸缩按钮位于标题栏 */
export function AppSidebar({
  page,
  onNavigate,
}: {
  page: Page;
  onNavigate: (page: Page) => void;
}) {
  const { t } = useTranslation();

  return (
    <Sidebar collapsible="icon">
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {NAV_ITEMS.map((item) => (
                <SidebarMenuItem key={item.page}>
                  <SidebarMenuButton
                    isActive={page === item.page}
                    tooltip={t(item.labelKey)}
                    onClick={() => onNavigate(item.page)}
                  >
                    <item.icon />
                    <span>{t(item.labelKey)}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
