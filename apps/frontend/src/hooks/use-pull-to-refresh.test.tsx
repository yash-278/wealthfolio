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
