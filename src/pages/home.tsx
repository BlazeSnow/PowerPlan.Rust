import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Copy, RefreshCw, Zap } from "lucide-react";
import { toast } from "sonner";
import {
  clearSavedUltimate,
  copyPlan,
  duplicateUltimate,
  getSettings,
  listPlans,
  openPowerOptions,
  setActivePlan,
  type AppSettings,
  type PlanInfo,
} from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import { PageHeader } from "@/components/page-header";
import {
  buildCopyPlanName,
  resolveUltimateState,
  type UltimateState,
} from "@/lib/plan";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";

export function HomePage() {
  const { t } = useTranslation();
  const [plans, setPlans] = useState<PlanInfo[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [copyTarget, setCopyTarget] = useState<PlanInfo | null>(null);
  const [copyName, setCopyName] = useState("");
  const [copyOpen, setCopyOpen] = useState(false);

  const refresh = useCallback(
    async (force = false) => {
      try {
        const [planList, appSettings] = await Promise.all([
          listPlans(force),
          getSettings(),
        ]);
        setPlans(planList);
        setSettings(appSettings);
      } catch (error) {
        showCommandError(t, error, "Main.Status.RefreshFailed");
      }
    },
    [t],
  );

  useEffect(() => {
    void refresh();
    // 托盘切换计划后通知前端刷新
    const unlistenPromise = listen("plans-changed", () => void refresh());
    return () => {
      void unlistenPromise.then((fn) => fn());
    };
  }, [refresh]);

  const switchPlan = useCallback(
    async (guid: string) => {
      try {
        await setActivePlan(guid);
        toast.success(
          t("Main.Status.SwitchSuccess", {
            0: plans.find((p) => p.guid === guid)?.name ?? guid,
          }),
        );
      } catch (error) {
        showCommandError(t, error, "Main.Status.SwitchFailed");
      }
      await refresh();
    },
    [t, plans, refresh],
  );

  const savedGuid = settings?.ultimatePerformancePlanGuid ?? null;
  // 卓越性能卡片三态（exists 隐藏卡片 / hidden 提供激活 / missing 提供创建）
  const ultimateState = resolveUltimateState(plans, savedGuid);

  const activateSavedUltimate = useCallback(async () => {
    if (!savedGuid) return;
    try {
      await setActivePlan(savedGuid);
      toast.success(t("Main.Status.UltimateActivated"));
    } catch (error) {
      // 激活失败说明计划已被删除，清空储存的 UUID（旧版 MainPage 行为）
      await clearSavedUltimate().catch(() => {});
      showCommandError(t, error, "Main.Status.UltimateActivateFailed");
    }
    await refresh(true);
  }, [t, savedGuid, refresh]);

  const createUltimate = useCallback(async () => {
    try {
      await duplicateUltimate();
      toast.success(t("Main.Status.UltimateCreated"));
    } catch (error) {
      showCommandError(t, error, "Main.Status.UltimateCreateFailed");
    }
    await refresh(true);
  }, [t, refresh]);

  const openCopyDialog = useCallback(
    (plan: PlanInfo) => {
      setCopyTarget(plan);
      setCopyName(
        buildCopyPlanName(plan.name, t("Main.DefaultPlanName"), t("Main.CopySuffix")),
      );
      setCopyOpen(true);
    },
    [t],
  );

  const submitCopy = useCallback(async () => {
    if (!copyTarget) return;
    const newName = copyName.trim();
    if (!newName) {
      toast.error(t("Main.Status.CopyNameEmpty"));
      return;
    }
    try {
      await copyPlan(copyTarget.guid, newName);
      toast.success(t("Main.Status.CopySuccess", { 0: newName }));
      await refresh(true);
    } catch (error) {
      showCommandError(t, error, "Main.Status.CopyFailed");
    }
  }, [t, copyTarget, copyName, refresh]);

  const activePlan = plans.find((p) => p.isActive) ?? null;

  return (
    <>
      <PageHeader titleKey="Shell.Home" />
      <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
      {ultimateState !== "exists" && (
        <UltimateCard
          state={ultimateState}
          onActivate={() => void activateSavedUltimate()}
          onCreate={() => void createUltimate()}
        />
      )}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>{t("Main.PlanPickerTitle")}</CardTitle>
            <Button variant="ghost" size="sm" onClick={() => void refresh(true)}>
              <RefreshCw className="size-4" />
              {t("Main.RefreshPlansButton")}
            </Button>
          </div>
          <CardDescription>{t("Main.DeletePlanHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          <RadioGroup
            value={activePlan?.guid ?? ""}
            onValueChange={(value) => void switchPlan(value)}
          >
            {plans.map((plan) => (
              <div key={plan.guid} className="flex items-center gap-1">
                <Label className="flex flex-1 cursor-pointer items-center gap-3 rounded-lg border p-3 font-normal">
                  <RadioGroupItem value={plan.guid} />
                  {plan.name}
                </Label>
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-9 shrink-0 text-muted-foreground"
                  onClick={() => openCopyDialog(plan)}
                >
                  <Copy className="size-4" />
                  <span className="sr-only">{t("Main.CopyPlanButton")}</span>
                </Button>
              </div>
            ))}
          </RadioGroup>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>{t("Main.PowerOptions")}</CardTitle>
          <CardDescription>{t("Main.PowerOptionsDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => void openPowerOptions()}>
            {t("Settings.Tools.OpenButton")}
          </Button>
        </CardContent>
      </Card>
      <StatusCard activePlanName={activePlan?.name ?? null} />
      <AlertDialog open={copyOpen} onOpenChange={setCopyOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("Main.CopyDialogTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("Main.CopyDialogPlaceholder")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <Input
            value={copyName}
            onChange={(e) => setCopyName(e.target.value)}
            placeholder={t("Main.CopyDialogPlaceholder")}
          />
          <AlertDialogFooter>
            <AlertDialogCancel>{t("Main.CopyDialogCancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void submitCopy()}>
              {t("Main.CopyDialogConfirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      </div>
    </>
  );
}

function UltimateCard({
  state,
  onActivate,
  onCreate,
}: {
  state: UltimateState;
  onActivate: () => void;
  onCreate: () => void;
}) {
  const { t } = useTranslation();
  const hidden = state === "hidden";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Zap className="size-4 text-amber-500" />
          {t(hidden ? "Main.UltimateHiddenTitle" : "Main.UltimateMissingTitle")}
        </CardTitle>
        <CardDescription>
          {t(hidden ? "Main.UltimateHiddenMessage" : "Main.UltimateMissingMessage")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        {hidden ? (
          <Button onClick={() => void onActivate()}>
            {t("Main.ActivateUltimateButton")}
          </Button>
        ) : (
          <Button onClick={() => void onCreate()}>
            {t("Main.CreateUltimateButton")}
          </Button>
        )}
      </CardContent>
    </Card>
  );
}

/** 状态：当前计划名称与当前时间；定时器仅在页面可见时运行 */
function StatusCard({ activePlanName }: { activePlanName: string | null }) {
  const { t } = useTranslation();
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    let timer: number | undefined;
    const start = () => {
      timer = window.setInterval(() => setNow(new Date()), 1000);
    };
    const stop = () => {
      if (timer !== undefined) window.clearInterval(timer);
    };
    const onVisibility = () => {
      if (document.hidden) {
        stop();
      } else {
        setNow(new Date());
        start();
      }
    };
    if (!document.hidden) start();
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      stop();
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  return (
    <Card>
      <CardContent className="flex items-center justify-between text-sm">
        <span>{activePlanName ?? t("Main.StatusWaiting")}</span>
        <span className="text-muted-foreground tabular-nums">
          {now.toLocaleTimeString()}
        </span>
      </CardContent>
    </Card>
  );
}
