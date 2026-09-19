import { useTranslation } from "react-i18next";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";

/** 内容区顶部行：侧边栏伸缩按钮 + 页面标题（不设独立顶栏） */
export function PageHeader({ titleKey }: { titleKey: string }) {
  const { t } = useTranslation();

  return (
    <div className="mb-4 flex items-center gap-2">
      <SidebarTrigger />
      <Separator
        orientation="vertical"
        className="data-[orientation=vertical]:h-4"
      />
      <h1 className="text-sm font-medium">{t(titleKey)}</h1>
    </div>
  );
}
