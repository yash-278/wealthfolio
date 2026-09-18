import { usePlatform } from "@/hooks/use-platform";
import { invoke, addPluginListener } from "@tauri-apps/api/core";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";

export interface NativeGlassItem {
  id: string;
  title: string;
  symbol: string;
  selected?: boolean;
  children?: NativeGlassItem[];
}

export function nativeNavigationSymbol(route: string): string {
  const symbols: Record<string, string> = {
    "quick-add": "plus",
    dashboard: "chart.xyaxis.line",
    insights: "chart.bar.xaxis",
    holdings: "briefcase",
    activities: "list.bullet.rectangle",
    goals: "target",
    assistant: "sparkles",
    settings: "gearshape",
    connect: "arrow.triangle.2.circlepath",
    investments: "chart.line.uptrend.xyaxis",
    "net-worth": "wallet.bifold",
    spending: "banknote",
  };
  return symbols[route.replace(/^\//, "").split("/")[0]] ?? "square.grid.2x2";
}

/** Retains the web layout while iOS renders and handles the actual controls. */
export function NativeGlassControls({
  items,
  onAction,
  children,
  labels = false,
  className,
}: {
  items: NativeGlassItem[];
  onAction: (id: string) => void;
  children: ReactNode;
  labels?: boolean;
  className?: string;
}) {
  const { isIOS, isTauri } = usePlatform();
  const id = useId();
  const anchor = useRef<HTMLDivElement>(null);
  const action = useRef(onAction);
  action.current = onAction;
  const [native, setNative] = useState(false);
  const itemJSON = JSON.stringify(
    items.map((item) => ({
      ...item,
      selected: item.selected ?? false,
      children: item.children?.map((child) => ({ ...child, selected: child.selected ?? false })),
    })),
  );

  const state = useRef({ itemJSON, labels });
  state.current = { itemJSON, labels };
  const refresh = useRef<(() => void) | undefined>(undefined);
  useEffect(() => refresh.current?.(), [itemJSON, labels]);

  useEffect(() => {
    if (!isIOS || !isTauri) return;
    const controlId = `${id}-${crypto.randomUUID()}`;
    let disposed = false;
    let frame = 0;
    let previous = "";
    let queue = Promise.resolve();
    let unlisten: (() => Promise<void>) | undefined;
    const update = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const element = anchor.current;
        if (disposed || !element) return;
        const rect = element.getBoundingClientRect();
        const modal = document.querySelector('[role="dialog"], [role="alertdialog"]');
        const editing = document.activeElement?.matches(
          'input, textarea, [contenteditable="true"]',
        );
        const payload = JSON.stringify({
          id: controlId,
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
          labels: state.current.labels,
          items: JSON.parse(state.current.itemJSON),
          dark: document.documentElement.classList.contains("dark"),
          visible:
            !modal &&
            !editing &&
            rect.width > 0 &&
            rect.height > 0 &&
            rect.top >= 0 &&
            rect.bottom <= window.innerHeight,
        });
        if (payload === previous) return;
        previous = payload;
        queue = queue.then(async () => {
          if (disposed) return;
          try {
            const result = await invoke<{ supported: boolean }>(
              "plugin:native-glass|update",
              JSON.parse(payload),
            );
            if (!disposed) setNative(result.supported);
          } catch {
            if (!disposed) setNative(false);
          }
        });
      });
    };
    refresh.current = update;
    void addPluginListener<{ control: string; item: string }>("native-glass", "action", (event) => {
      if (!disposed && event.control === controlId) action.current(event.item);
    })
      .then((listener) => {
        if (disposed) {
          void listener.unregister();
          return;
        }
        unlisten = () => listener.unregister();
        update();
      })
      .catch(() => setNative(false));
    const resize = new ResizeObserver(update);
    if (anchor.current) resize.observe(anchor.current);
    const mutations = new MutationObserver(update);
    mutations.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["data-state", "style", "aria-hidden"],
    });
    mutations.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    window.addEventListener("resize", update);
    document.addEventListener("scroll", update, true);
    document.addEventListener("focusin", update);
    document.addEventListener("focusout", update);
    return () => {
      disposed = true;
      refresh.current = undefined;
      cancelAnimationFrame(frame);
      resize.disconnect();
      mutations.disconnect();
      window.removeEventListener("resize", update);
      document.removeEventListener("scroll", update, true);
      document.removeEventListener("focusin", update);
      document.removeEventListener("focusout", update);
      void unlisten?.();
      void queue
        .then(() => invoke("plugin:native-glass|remove", { id: controlId }))
        .catch(() => undefined);
    };
  }, [id, isIOS, isTauri]);

  return (
    <div
      ref={anchor}
      className={className}
      style={native ? { minHeight: 44, minWidth: 44 } : undefined}
    >
      <div
        aria-hidden={native || undefined}
        inert={native || undefined}
        style={native ? { opacity: 0, pointerEvents: "none" } : undefined}
      >
        {children}
      </div>
    </div>
  );
}
