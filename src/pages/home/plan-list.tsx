import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Copy, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { copyPlan, type PlanInfo } from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import { buildCopyPlanName } from "@/lib/plan";
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
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";

/** 电源计划列表卡 + 复制计划对话框（复制状态自管，完成后回调刷新） */
export function PlanList({
  plans,
  activeGuid,
  onSwitch,
  onRefresh,
}: {
  plans: PlanInfo[];
  activeGuid: string;
  onSwitch: (guid: string) => Promise<void>;
  onRefresh: (force?: boolean) => Promise<void>;
}) {
  const { t } = useTranslation();
  const [copyTarget, setCopyTarget] = useState<PlanInfo | null>(null);
  const [copyName, setCopyName] = useState("");
  const [copyOpen, setCopyOpen] = useState(false);

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
      await onRefresh(true);
    } catch (error) {
      showCommandError(t, error, "Main.Status.CopyFailed");
    }
  }, [t, copyTarget, copyName, onRefresh]);

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("Main.PlanPickerTitle")}</CardTitle>
          <CardDescription>{t("Main.DeletePlanHint")}</CardDescription>
          <CardAction>
            <Button variant="ghost" size="sm" onClick={() => void onRefresh(true)}>
              <RefreshCw className="size-4" />
              {t("Main.RefreshPlansButton")}
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          <RadioGroup
            value={activeGuid}
            onValueChange={(value) => void onSwitch(value)}
          >
            {plans.map((plan) => (
              <div key={plan.guid} className="flex items-center gap-1">
                <Label className="flex flex-1 cursor-pointer items-center gap-3 rounded-lg border p-3 font-normal">
                  <RadioGroupItem value={plan.guid} />
                  {/* 对齐旧版：名称在上，GUID 以小字附下（悬停可看完整值） */}
                  <span className="flex min-w-0 flex-col gap-0.5">
                    <span className="text-sm font-medium leading-tight">
                      {plan.name}
                    </span>
                    <span
                      title={plan.guid}
                      className="truncate text-xs leading-tight text-muted-foreground tabular-nums"
                    >
                      {plan.guid}
                    </span>
                  </span>
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
    </>
  );
}
