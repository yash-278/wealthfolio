import SwiftUI

@main struct WealthfolioNativeApp: App {
    @State private var model = AppModel()
    @Environment(\.scenePhase) private var phase
    var body: some Scene {
        WindowGroup {
            Group {
                if model.loaded && model.currency.isEmpty { SetupView() }
                else if model.loaded { AppTabs() }
                else if let error = model.startupError {
                    ContentUnavailableView("Unable to open portfolio", systemImage: "exclamationmark.triangle", description: Text(error))
                } else { ProgressView("Opening your portfolio…") }
            }
            .environment(model)
            .tint(.accentColor)
            .task { await model.start() }
            .task(id: phase) {
                guard phase == .active else { return }
                while !Task.isCancelled {
                    await model.synchronize()
                    do { try await Task.sleep(for: .seconds(30)) } catch { return }
                }
            }
            .task(id: model.ready && phase == .active) {
                guard model.ready && phase == .active else { return }
                while !Task.isCancelled {
                    if let events = try? await model.engine.request("/native/events"), !events.values.isEmpty {
                        await model.refresh()
                    }
                    do { try await Task.sleep(for: .seconds(1)) } catch { return }
                }
            }
            .alert("Could not complete the action", isPresented: Binding(
                get: { model.error != nil }, set: { if !$0 { model.error = nil } })) {
                Button("OK") { model.error = nil }
            } message: { Text(model.error ?? "") }
        }
    }
}

struct AppTabs: View {
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        TabView {
            Tab("Dashboard", systemImage: "chart.xyaxis.line") { NavigationStack { DashboardView() } }
            Tab("Activities", systemImage: "list.bullet.rectangle") { NavigationStack { ActivitiesView() } }
            Tab("Insights", systemImage: "chart.bar.xaxis") { NavigationStack { InsightsView() } }
            Tab("Assistant", systemImage: "sparkles") { NavigationStack { AssistantView() } }
            Tab("More", systemImage: "square.grid.2x2") { NavigationStack { MoreView() } }
        }.tint(scheme == .dark ? Color(red: 0.62, green: 0.86, blue: 0.83) : Color(red: 0.16, green: 0.46, blue: 0.44))
    }
}
