import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MobileNavigationContainer } from "@/pages/layouts/mobile-navigation-container";
import { SwipablePage } from "@/components/page/swipable-page";
import { createRoot } from "react-dom/client";
import { MemoryRouter, Outlet, Route, Routes } from "react-router-dom";
import { ApplicationShell, PageScrollContainer } from "@wealthfolio/ui";
import QuickAddPage from "@/features/quick-add/quick-add-page";
import { QuickAddReviewPage } from "@/features/quick-add/review-panel";
import CaptureSettingsPanel from "@/features/quick-add/settings-panel";
import "@/globals.css";

// Real page components and shared shell, with synthetic HTTP data supplied by
// the browser test. No production server or financial writes are involved.
const queryClient = new QueryClient();

function Fixture() {
  const params = new URLSearchParams(location.search);
  const mobile = matchMedia("(max-width: 767px)").matches;
  return (
    <MemoryRouter initialEntries={[params.get("route") ?? "/quick-add"]}>
      <ApplicationShell className="app-shell h-dvh min-h-0 overflow-hidden">
        <main className="relative flex min-h-0 w-full min-w-0 flex-1 flex-col overflow-x-hidden">
          <Routes>
            <Route
              element={
                mobile ? (
                  <MobileNavigationContainer />
                ) : (
                  <PageScrollContainer>
                    <Outlet />
                  </PageScrollContainer>
                )
              }
            >
              <Route
                path="/dashboard-layout"
                element={
                  <SwipablePage
                    defaultView="investments"
                    withPadding={false}
                    withMobileNavOffset={false}
                    views={[
                      {
                        value: "investments",
                        label: "Investments",
                        content: (
                          <div className="bg-green-100 pb-[var(--mobile-nav-total-offset)]">
                            <div className="h-[900px]">Synthetic dashboard</div>
                            <button data-dashboard-end>Last dashboard action</button>
                          </div>
                        ),
                      },
                      {
                        value: "net-worth",
                        label: "Net worth",
                        content: <div className="h-[1600px]">Longer inactive view</div>,
                      },
                    ]}
                  />
                }
              />
              <Route path="/quick-add" element={<QuickAddPage />} />
              <Route path="/quick-add/review" element={<QuickAddReviewPage />} />
              <Route
                path="/settings/quick-add"
                element={
                  <div className="p-4 pb-[var(--mobile-nav-total-offset)]">
                    <CaptureSettingsPanel />
                  </div>
                }
              />
            </Route>
          </Routes>
        </main>
        {mobile && (
          <nav
            aria-label="Bottom navigation"
            className="bg-background fixed inset-x-4 bottom-[var(--mobile-nav-bottom-offset)] h-[var(--mobile-nav-ui-height)] rounded-xl border"
          >
            Bottom navigation
          </nav>
        )}
      </ApplicationShell>
    </MemoryRouter>
  );
}

createRoot(document.getElementById("root")!).render(
  <QueryClientProvider client={queryClient}>
    <Fixture />
  </QueryClientProvider>,
);
