import { useTranslation } from "react-i18next";
import { NativeGlassControls } from "@/components/native-glass-controls";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Icons } from "@wealthfolio/ui/components/ui/icons";
import { useBalancePrivacy } from "@/hooks/use-balance-privacy";
import { cn } from "@/lib/utils";

interface PrivacyToggleProps {
  className?: string;
}

export function PrivacyToggle({ className }: PrivacyToggleProps) {
  const { t } = useTranslation();
  const { isBalanceHidden, toggleBalanceVisibility } = useBalancePrivacy();

  return (
    <NativeGlassControls
      items={[
        {
          id: "privacy",
          title: t(
            isBalanceHidden ? "common:component.show_balance" : "common:component.hide_balance",
          ),
          symbol: isBalanceHidden ? "eye" : "eye.slash",
        },
      ]}
      onAction={toggleBalanceVisibility}
    >
      <Button
        variant="secondary"
        size="icon-xs"
        className={cn("bg-secondary/50 rounded-full", className)}
        onClick={(e) => {
          e.stopPropagation();
          toggleBalanceVisibility();
        }}
      >
        {isBalanceHidden ? <Icons.Eye className="size-5" /> : <Icons.EyeOff className="size-5" />}
      </Button>
    </NativeGlassControls>
  );
}
