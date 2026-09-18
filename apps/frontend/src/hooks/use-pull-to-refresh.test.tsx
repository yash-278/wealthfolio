import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { usePullToRefresh } from "./use-pull-to-refresh";

vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
}));
vi.mock("@/hooks/use-calculate-portfolio", () => ({
  useUpdatePortfolioMutation: () => ({ mutateAsync: vi.fn() }),
}));
vi.mock("@/hooks/use-haptic-feedback", () => ({
  useHapticFeedback: () => ({ triggerHaptic: vi.fn() }),
}));

function Fixture({ refresh }: { refresh: () => Promise<void> }) {
  const [, handlers, state] = usePullToRefresh({ onRefresh: refresh });
  return (
    <div data-testid="scroll" {...handlers}>
      <div data-ptr-content>
        <textarea aria-label="Text" />
        <span data-testid="activated">{String(state.isActivated)}</span>
      </div>
    </div>
  );
}
const touches = (y: number) => ({ touches: [{ clientX: 100, clientY: y }] });

it("cancels an activated pull without refreshing or disabling future scrolling", () => {
  const refresh = vi.fn().mockResolvedValue(undefined);
  render(<Fixture refresh={refresh} />);
  const container = screen.getByTestId("scroll");
  fireEvent.touchStart(container, touches(0));
  fireEvent.touchMove(container, touches(300));
  expect(screen.getByTestId("activated")).toHaveTextContent("true");
  expect(container.style.touchAction).not.toBe("none");
  fireEvent.touchCancel(container);
  expect(screen.getByTestId("activated")).toHaveTextContent("false");
  expect(refresh).not.toHaveBeenCalled();
  fireEvent.touchStart(container, touches(0));
  fireEvent.touchMove(container, touches(300));
  fireEvent.touchEnd(container);
  expect(refresh).toHaveBeenCalledOnce();
});

it("leaves text editing and an already scrolled page to native touch handling", () => {
  const refresh = vi.fn().mockResolvedValue(undefined);
  render(<Fixture refresh={refresh} />);
  const container = screen.getByTestId("scroll");
  fireEvent.touchStart(screen.getByLabelText("Text"), touches(0));
  fireEvent.touchMove(container, touches(300));
  fireEvent.touchEnd(container);
  container.scrollTop = 100;
  fireEvent.touchStart(container, touches(0));
  container.scrollTop = 0;
  fireEvent.touchMove(container, touches(300));
  fireEvent.touchEnd(container);
  expect(refresh).not.toHaveBeenCalled();
});

it.each(["upward flick", "scrolled page", "nested scroller", "text editing"])(
  "does not animate content or reset styles after %s",
  (gesture) => {
    const refresh = vi.fn().mockResolvedValue(undefined);
    render(<Fixture refresh={refresh} />);
    const container = screen.getByTestId("scroll");
    const content = container.querySelector<HTMLElement>("[data-ptr-content]")!;
    container.style.touchAction = "pan-y";
    container.style.transform = "translateZ(0)";
    container.style.transition = "opacity 100ms";
    const before = container.style.cssText;
    let target: HTMLElement = container;
    if (gesture === "scrolled page") container.scrollTop = 100;
    if (gesture === "nested scroller") {
      content.scrollTop = 100;
      target = content;
    }
    if (gesture === "text editing") target = screen.getByLabelText("Text");
    fireEvent.touchStart(target, touches(300));
    fireEvent.touchMove(target, touches(gesture === "upward flick" ? 100 : 500));
    fireEvent.touchEnd(target);
    expect(content.style.cssText).toBe("");
    expect(container.style.cssText).toBe(before);
    expect(refresh).not.toHaveBeenCalled();
  },
);
