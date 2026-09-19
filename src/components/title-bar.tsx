import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SidebarTrigger } from "@/components/ui/sidebar";

const appWindow = getCurrentWindow();

/** 自绘标题栏：拖拽区 + 伸缩侧边栏按钮 + 软件名称/简介 + 窗口控制 */
export function TitleBar() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const unlistenPromise = appWindow.onResized(() => {
      void appWindow.isMaximized().then(setMaximized);
    });
    void appWindow.isMaximized().then(setMaximized);
    return () => {
      void unlistenPromise.then((fn) => fn());
    };
  }, []);

  return (
    <header
      data-tauri-drag-region
      className="flex h-12 shrink-0 select-none items-center gap-1 border-b px-2"
    >
      <SidebarTrigger />
      <div
        data-tauri-drag-region
        className="flex flex-1 flex-col justify-center px-2 leading-tight"
      >
        <span className="text-sm font-semibold">
          {t("AppTitleBar.Title")}
        </span>
        <span className="text-xs text-muted-foreground">
          {t("AppTitleBar.Subtitle")}
        </span>
      </div>
      <div className="flex items-center">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-10 rounded-none"
          onClick={() => void appWindow.minimize()}
        >
          <Minus />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-10 rounded-none"
          onClick={() => void appWindow.toggleMaximize()}
        >
          {maximized ? <Square className="size-3.5" /> : <Square />}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-10 rounded-none hover:bg-destructive/10 hover:text-destructive"
          onClick={() => void appWindow.close()}
        >
          <X />
        </Button>
      </div>
    </header>
  );
}
