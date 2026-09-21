import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getVersion } from "@tauri-apps/api/app";
import { Info, Wrench } from "lucide-react";
import { toast } from "sonner";
import i18n, { resolveLanguage } from "@/i18n";
import {
  getSettings,
  setAutoStart,
  setLanguage as persistLanguage,
  setLaunchToTray,
  setTray,
  type AppSettings,
} from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import { PageHeader } from "@/components/page-header";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LinkCard, SwitchRow } from "./rows";
import { RestoreCard } from "./restore-card";

const WEBSITE_URL = "https://powerplan.blazesnow.com/";
const REPOSITORY_URL = "https://github.com/BlazeSnow/PowerPlan.Rust";

/**
 * 开机自启动的系统侧状态与开关不一致时经 toast 提示（复用主页操作反馈形式）：
 * - 开关期望开启但被用户/策略禁用 → 警告（存在需要处理的差异）
 * - 环境不支持 → 提示
 * 正常 enabled/disabled 与开关一致，不打扰。
 */
function notifyAutostartMismatch(
  t: (key: string) => string,
  value: AppSettings,
) {
  const title = t("Settings.AutoStart.Title");
  if (
    value.autoStartEnabled &&
    value.autoStartState === "disabled_by_user"
  ) {
    toast.warning(title, {
      description: t("Settings.AutoStart.StateDisabledByUser"),
    });
  } else if (
    value.autoStartEnabled &&
    value.autoStartState === "disabled_by_policy"
  ) {
    toast.warning(title, {
      description: t("Settings.AutoStart.StateDisabledByPolicy"),
    });
  } else if (value.autoStartState === "unsupported") {
    toast.info(title, {
      description: t("Settings.AutoStart.StateUnsupported"),
    });
  }
}

// 语言名称以各自语言显示，不翻译
const LANGUAGES: { value: string; label: string }[] = [
  { value: "zh-Hans", label: "简体中文" },
  { value: "zh-Hant", label: "繁體中文" },
  { value: "en", label: "English" },
  { value: "fr", label: "Français" },
  { value: "it", label: "Italiano" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
];

export function SettingsPage() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [version, setVersion] = useState("");

  useEffect(() => {
    void getSettings()
      .then((value) => {
        setSettings(value);
        notifyAutostartMismatch(t, value);
      })
      .catch(() => {});
    void getVersion().then(setVersion).catch(() => {});
    // 仅挂载时提示一次，语言切换不重复打扰
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const changeLanguage = async (value: string) => {
    if (!settings) return;
    setSettings({ ...settings, language: value });
    // 语言即时生效：前端立即切换，后端托盘菜单由命令内重建，无需重启
    void i18n.changeLanguage(resolveLanguage(value));
    try {
      const updated = await persistLanguage(value);
      setSettings(updated);
    } catch (error) {
      showCommandError(t, error, "Settings.SaveFailed");
    }
  };

  const toggleAutoStart = async (value: boolean) => {
    if (!settings) return;
    setSettings({ ...settings, autoStartEnabled: value });
    try {
      const updated = await setAutoStart(value);
      setSettings(updated);
    } catch (error) {
      setSettings({ ...settings, autoStartEnabled: !value });
      showCommandError(t, error, "App.Status.StartupSettingFailed");
    }
  };

  const toggleTray = async (value: boolean) => {
    if (!settings) return;
    setSettings({ ...settings, trayEnabled: value });
    try {
      const updated = await setTray(value);
      setSettings(updated);
    } catch (error) {
      setSettings({ ...settings, trayEnabled: !value });
      showCommandError(t, error, "Settings.SaveFailed");
    }
  };

  const toggleLaunchToTray = async (value: boolean) => {
    if (!settings) return;
    setSettings({ ...settings, launchToTray: value });
    try {
      const updated = await setLaunchToTray(value);
      setSettings(updated);
    } catch (error) {
      setSettings({ ...settings, launchToTray: !value });
      showCommandError(t, error, "Settings.SaveFailed");
    }
  };

  return (
    <>
      <PageHeader titleKey="Shell.Settings" />
      <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Wrench className="size-4" />
              {t("Settings.Tools.Title")}
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-6">
            <div className="flex items-center justify-between gap-4">
              <div className="flex flex-col gap-1">
                <span className="text-sm font-medium">
                  {t("Settings.Language.Title")}
                </span>
                <span className="text-sm text-muted-foreground">
                  {t("Settings.Language.Desc")}
                </span>
              </div>
              <Select
                value={settings?.language ?? "auto"}
                onValueChange={(value) => void changeLanguage(value)}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto">
                    {t("Settings.Language.Automatic")}
                  </SelectItem>
                  {LANGUAGES.map((language) => (
                    <SelectItem key={language.value} value={language.value}>
                      {language.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <SwitchRow
              title={t("Settings.AutoStart.Title")}
              description={t("Settings.AutoStart.Desc")}
              checked={settings?.autoStartEnabled ?? false}
              disabled={!settings || settings.autoStartState === "unsupported"}
              onCheckedChange={(value) => void toggleAutoStart(value)}
            />
            <SwitchRow
              title={t("Settings.Tray.Title")}
              description={t("Settings.Tray.Desc")}
              checked={settings?.trayEnabled ?? false}
              disabled={!settings}
              onCheckedChange={(value) => void toggleTray(value)}
            />
            <SwitchRow
              title={t("Settings.LaunchToTray.Title")}
              description={t("Settings.LaunchToTray.Desc")}
              checked={settings?.launchToTray ?? false}
              disabled={!settings || !settings.trayEnabled}
              onCheckedChange={(value) => void toggleLaunchToTray(value)}
            />
          </CardContent>
        </Card>
        <RestoreCard />
        <LinkCard
          title={t("Settings.Tools.Website")}
          description={t("Settings.Tools.WebsiteDesc")}
          url={WEBSITE_URL}
          openLabel={t("Settings.Tools.OpenButton")}
        />
        <LinkCard
          title={t("Settings.Tools.Repository")}
          description={t("Settings.Tools.RepositoryDesc")}
          url={REPOSITORY_URL}
          openLabel={t("Settings.Tools.OpenButton")}
        />
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Info className="size-4" />
              {t("Settings.AppVersion.Title")}
            </CardTitle>
            <CardDescription>{t("Settings.AppVersion.Desc")}</CardDescription>
            <CardAction>
              <span className="text-sm tabular-nums text-muted-foreground">
                {version}
              </span>
            </CardAction>
          </CardHeader>
        </Card>
      </div>
    </>
  );
}
