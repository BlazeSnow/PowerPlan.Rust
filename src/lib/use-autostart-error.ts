import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

/** 托盘开机自启动切换失败提示：后端打开主窗口并 emit `autostart-error`
 * （payload 为前端文案键 + 插值参数），经软件内自绘 toast 提示——
 * 不受系统通知设置影响。 */
export function useAutostartErrorToast() {
  const { t } = useTranslation();

  useEffect(() => {
    const unlistenPromise = listen<{
      key: string;
      args?: Record<string, string>;
    }>("autostart-error", (e) => {
      toast.error(t(e.payload.key, e.payload.args));
    });
    return () => {
      void unlistenPromise.then((fn) => fn());
    };
  }, [t]);
}
