import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { takeOpenSettingsRequest } from "@/lib/api";

/** 托盘「打开设置」导航：窗口存活时后端 emit `open-settings` 即时切换；
 * webview 销毁重建场景下事件早于监听注册，挂载时消费后端挂起标记兜底。
 * 事件与挂载两条路径都会消费标记，避免陈旧标记在无关重开窗口时误导航。 */
export function useOpenSettings(onOpen: () => void) {
  useEffect(() => {
    void takeOpenSettingsRequest().then((open) => {
      if (open) onOpen();
    });
    const unlistenPromise = listen("open-settings", () => {
      void takeOpenSettingsRequest();
      onOpen();
    });
    return () => {
      void unlistenPromise.then((fn) => fn());
    };
  }, [onOpen]);
}
