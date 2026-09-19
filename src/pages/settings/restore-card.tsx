import { useTranslation } from "react-i18next";
import { RotateCcw } from "lucide-react";
import { toast } from "sonner";
import { restoreDefaults } from "@/lib/api";
import { showCommandError } from "@/lib/command-error";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

/** 恢复电源计划卡片：确认弹窗 + 调用后端恢复（需管理员权限） */
export function RestoreCard() {
  const { t } = useTranslation();

  const restore = async () => {
    try {
      await restoreDefaults();
      toast.success(t("Settings.RestoreDialog.SuccessTitle"), {
        description: t("Settings.RestoreDialog.SuccessMessage"),
      });
    } catch (error) {
      showCommandError(t, error, "Settings.RestoreDialog.FailedMessage");
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <RotateCcw className="size-4" />
          {t("Settings.Tools.RestorePowerPlans")}
        </CardTitle>
        <CardDescription>
          {t("Settings.Tools.RestorePowerPlansDesc")}
        </CardDescription>
        <CardAction>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="outline">
                {t("Settings.Tools.RestoreButton")}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t("Settings.RestoreConfirmDialog.Title")}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t("Settings.RestoreConfirmDialog.Message")}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>
                  {t("Settings.RestoreConfirmDialog.Cancel")}
                </AlertDialogCancel>
                <AlertDialogAction
                  variant="destructive"
                  onClick={() => void restore()}
                >
                  {t("Settings.RestoreConfirmDialog.Confirm")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </CardAction>
      </CardHeader>
    </Card>
  );
}
