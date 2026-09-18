import { NativeGlassControls, nativeNavigationSymbol } from "@/components/native-glass-controls";
import { LiquidGlass } from "@/components/liquid-glass";
import { SyncStatusIcon } from "@/features/wealthfolio-connect/components/sync-status-icon";
import { useAggregatedSyncStatus } from "@/features/wealthfolio-connect/hooks";
import { useHapticFeedback } from "@/hooks/use-haptic-feedback";
import { cn } from "@/lib/utils";
import { Icons, Sheet, SheetContent, SheetTitle } from "@wealthfolio/ui";
import { motion } from "motion/react";
import { useCallback, useId, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { type NavLink, type NavigationProps, isPathActive } from "./app-navigation";
import { resolveNavigationIcon } from "./navigation-icons";

interface MobileNavBarProps {
  navigation: NavigationProps;
}

export function MobileNavBar({ navigation }: MobileNavBarProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const { triggerHaptic } = useHapticFeedback();
  const uniqueId = useId();
  const { status: syncStatus } = useAggregatedSyncStatus();

  const containerClassName = "pointer-events-none fixed inset-x-0 bottom-0 z-50";

  const handleNavigation = useCallback(
    (href: string, isActive: boolean) => {
      if (isActive) return;
      if (href === "#search") {
        document.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "k",
            code: "KeyK",
            metaKey: true,
            ctrlKey: true,
            bubbles: true,
            cancelable: true,
          }),
        );
        return;
      }
      triggerHaptic();
      navigate(href);
    },
    [triggerHaptic, navigate],
  );

  const renderIcon = useCallback((icon?: ReactNode) => resolveNavigationIcon(icon, "size-6"), []);

  const primaryItems = navigation?.primary ?? [];
  const secondaryItems = navigation?.secondary ?? [];
  const pinnedAddonItems = navigation?.pinnedAddons ?? [];
  const addonMenuItems = navigation?.addonMenuItems ?? navigation?.addons ?? [];
  const visibleRoutes = ["/dashboard", "/activities", "/insights", "/assistant"];
  const visibleItems = visibleRoutes.flatMap((href) =>
    primaryItems.filter((item) => item.href === href),
  );
  const addonItems = [...pinnedAddonItems, ...addonMenuItems];
  const standardMenuItems: NavLink[] = [
    ...primaryItems.filter((item) => !visibleRoutes.includes(item.href)),
    { title: t("common:search"), href: "#search", icon: <Icons.Search2 className="size-6" /> },
    ...secondaryItems,
    {
      title: t("common:connect"),
      href: "/connect",
      icon: <SyncStatusIcon status={syncStatus} className="size-6" />,
    },
  ];
  const moreItems = [...standardMenuItems, ...addonItems];
  const hasMenu = moreItems.length > 0;
  const columnCount = visibleItems.length + (hasMenu ? 1 : 0);

  return (
    <div className={containerClassName}>
      {/* Lift off bottom by the design gap while respecting safe area */}
      <div className="flex justify-center px-4 pb-[var(--mobile-nav-bottom-offset)]">
        <NativeGlassControls
          className="w-full"
          labels
          items={[
            ...visibleItems.map((item) => ({
              id: item.href,
              title: item.title,
              symbol:
                item.href === "#search" ? "magnifyingglass" : nativeNavigationSymbol(item.href),
              selected: isPathActive(location.pathname, item.href),
            })),
            ...(hasMenu
              ? [
                  {
                    id: "more",
                    title: t("common:layout.more"),
                    symbol: "square.grid.2x2",
                    children: moreItems.map((item) => ({
                      id: item.href,
                      title: item.title,
                      symbol:
                        item.href === "#search"
                          ? "magnifyingglass"
                          : nativeNavigationSymbol(item.href),
                    })),
                  },
                ]
              : []),
          ]}
          onAction={(id) => {
            if (id === "#search") {
              document.dispatchEvent(
                new KeyboardEvent("keydown", {
                  key: "k",
                  code: "KeyK",
                  metaKey: true,
                  ctrlKey: true,
                  bubbles: true,
                  cancelable: true,
                }),
              );
            } else handleNavigation(id, isPathActive(location.pathname, id));
          }}
        >
          <LiquidGlass
            variant="floating"
            intensity="subtle"
            className={cn(
              "pointer-events-auto w-full px-1 py-1",
              "h-[var(--mobile-nav-ui-height)]",
            )}
          >
            <nav
              aria-label={t("common:layout.primary_navigation")}
              className={cn("grid place-items-center gap-2")}
              style={{ gridTemplateColumns: `repeat(${columnCount}, minmax(0, 1fr))` }}
            >
              {visibleItems.map((item) => {
                const isActive = isPathActive(location.pathname, item.href);
                const isSearch = item.href === "#search";

                return (
                  <Link
                    to={item.href}
                    onClick={(e) => {
                      if (isSearch) {
                        e.preventDefault();
                        triggerHaptic();
                        const event = new KeyboardEvent("keydown", {
                          key: "k",
                          code: "KeyK",
                          keyCode: 75,
                          which: 75,
                          metaKey: true,
                          ctrlKey: true,
                          bubbles: true,
                          cancelable: true,
                        });
                        document.dispatchEvent(event);
                      } else {
                        handleNavigation(item.href, isActive);
                      }
                    }}
                    aria-label={item.title}
                    className="text-foreground relative z-10 flex h-14 w-full items-center justify-center rounded-full transition-colors"
                    key={item.href}
                    aria-current={isActive ? "page" : undefined}
                  >
                    {isActive && (
                      <motion.div
                        layoutId={`mobile-nav-indicator-${uniqueId}`}
                        className="absolute inset-0 -z-10 rounded-full border border-black/10 bg-black/5 shadow-sm dark:border-white/10 dark:bg-white/10"
                        initial={false}
                        transition={{
                          type: "spring",
                          stiffness: 400,
                          damping: 30,
                        }}
                      />
                    )}
                    <span
                      className="relative flex size-7 shrink-0 items-center justify-center outline-none"
                      aria-hidden="true"
                    >
                      {renderIcon(item.icon)}
                    </span>
                  </Link>
                );
              })}

              {hasMenu && (
                <button
                  onClick={() => {
                    triggerHaptic();
                    setMobileMenuOpen(true);
                  }}
                  aria-label={t("common:layout.more_options")}
                  className="text-foreground relative z-10 flex h-14 w-full items-center justify-center rounded-full transition-colors"
                >
                  {moreItems.some((item) => isPathActive(location.pathname, item.href)) && (
                    <motion.div
                      layoutId={`mobile-nav-indicator-${uniqueId}`}
                      className="absolute inset-0 -z-10 rounded-full border border-black/10 bg-black/5 shadow-sm dark:border-white/10 dark:bg-white/10"
                      initial={false}
                      transition={{
                        type: "spring",
                        stiffness: 400,
                        damping: 30,
                      }}
                    />
                  )}
                  <span
                    className="relative flex size-7 shrink-0 items-center justify-center outline-none"
                    aria-hidden="true"
                  >
                    <Icons.CirclesFour className="size-6" />
                  </span>
                </button>
              )}
            </nav>
          </LiquidGlass>
        </NativeGlassControls>
      </div>

      <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
        <SheetContent
          side="bottom"
          showCloseButton={false}
          className="bg-background inset-x-4 bottom-4 max-h-[min(82vh,720px)] overflow-hidden rounded-[2rem] border-0 px-0 pb-[calc(env(safe-area-inset-bottom,0px)+1.25rem)] pt-0 shadow-2xl"
        >
          <div className="bg-muted mx-auto mt-4 h-1.5 w-14 rounded-full" />
          <div className="flex items-center justify-between px-8 pb-4 pt-7">
            <SheetTitle className="text-2xl font-semibold">{t("common:layout.more")}</SheetTitle>
            <button
              type="button"
              onClick={() => setMobileMenuOpen(false)}
              className="bg-muted text-foreground hover:bg-muted/80 flex size-11 items-center justify-center rounded-full transition-colors"
              aria-label={t("common:layout.close_more_menu")}
            >
              <Icons.Close className="size-5" />
            </button>
          </div>

          <div className="scrollbar-hide max-h-[calc(min(82vh,720px)-7rem)] overflow-y-auto px-8">
            <div className="divide-border/70 divide-y">
              {standardMenuItems.map((item) => {
                const isActive = isPathActive(location.pathname, item.href);

                return (
                  <Link
                    key={item.href}
                    to={item.href}
                    onClick={() => {
                      handleNavigation(item.href, isActive);
                      setMobileMenuOpen(false);
                    }}
                    aria-current={isActive ? "page" : undefined}
                    className={cn(
                      "group flex h-16 items-center gap-4 transition-colors",
                      isActive ? "text-primary" : "text-foreground",
                    )}
                  >
                    <span className="flex size-7 shrink-0 items-center justify-center">
                      {renderIcon(item.icon)}
                    </span>
                    <span className="min-w-0 flex-1 truncate text-lg font-semibold">
                      {item.title}
                    </span>
                    <Icons.ChevronRight className="text-muted-foreground/50 size-5 shrink-0 transition-transform group-hover:translate-x-0.5" />
                  </Link>
                );
              })}
            </div>

            {addonItems.length > 0 && (
              <div className="pt-6">
                <div className="text-muted-foreground pb-3 text-xs font-semibold uppercase tracking-[0.35em]">
                  {t("common:addons")}
                </div>
                <div className="divide-border/70 divide-y">
                  {addonItems.map((item) => {
                    const isActive = isPathActive(location.pathname, item.href);

                    return (
                      <Link
                        key={item.href}
                        to={item.href}
                        onClick={() => {
                          handleNavigation(item.href, isActive);
                          setMobileMenuOpen(false);
                        }}
                        aria-current={isActive ? "page" : undefined}
                        className={cn(
                          "group flex h-16 items-center gap-4 transition-colors",
                          isActive ? "text-primary" : "text-foreground",
                        )}
                      >
                        <span className="flex size-7 shrink-0 items-center justify-center">
                          {renderIcon(item.icon)}
                        </span>
                        <span className="min-w-0 flex-1 truncate text-lg font-semibold">
                          {item.title}
                        </span>
                        <Icons.ChevronRight className="text-muted-foreground/50 size-5 shrink-0 transition-transform group-hover:translate-x-0.5" />
                      </Link>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
