import SwiftUI
import Charts

struct SpendingView: View {
    var embedded = false
    var dashboardHeader: AnyView? = nil
    var onReport: (JSONValue) -> Void = { _ in }
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var selectedDay: String?
    @Environment(AppModel.self) private var model
    @State private var month = Date()
    @State private var report: JSONValue = .null
    @State private var error: String?
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                dashboardHeader
                DatePicker("Month", selection: $month, displayedComponents: .date).font(.subheadline)
                if let error { Text(error).foregroundStyle(.red) }
                if !embedded { VStack(alignment: .leading, spacing: 8) {
                    Text("Spending").font(.subheadline).foregroundStyle(.secondary)
                    Text(money(report["current"]["outflow"], currency: model.currency, hidden: model.hideBalances))
                        .font(.system(.largeTitle, design: .rounded).weight(.semibold)).monospacedDigit()
                }
                }
                if !report["byDay"].values.isEmpty && !model.hideBalances {
                    dailyChart
                }
                VStack(spacing: 18) {
                    LabeledContent("Income", value: money(report["current"]["income"], currency: model.currency, hidden: model.hideBalances))
                    LabeledContent("Saved", value: money(report["current"]["saved"], currency: model.currency, hidden: model.hideBalances))
                    LabeledContent("Net", value: money(report["current"]["net"], currency: model.currency, hidden: model.hideBalances))
                }.font(.subheadline.weight(.medium)).monospacedDigit().padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                NavigationLink { SpendingSettingsView() } label: { Label("Spending accounts", systemImage: "slider.horizontal.3") }.buttonStyle(.glass)
            }.padding(24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).tint(palette.accent).leadingPageTitle("Spending", enabled: !embedded)
        .task(id: month) { await load() }.refreshable { await load() }
    }
    private var dailyChart: some View {
        let rows = report["byDay"].values
        let selected = rows.first { $0["date"].text == selectedDay }
        return VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text(selectedDay ?? "Daily spending").font(.subheadline)
                Spacer()
                if let selected { Text(money(selected["outflow"], currency: model.currency)).font(.subheadline.weight(.semibold)).monospacedDigit() }
            }
            Chart(Array(rows.enumerated()), id: \.offset) { _, row in
                if let amount = row["outflow"].decimal {
                    BarMark(x: .value("Day", row["date"].text), y: .value("Spending", NSDecimalNumber(decimal: amount).doubleValue))
                        .foregroundStyle(palette.accent).cornerRadius(3)
                        .opacity(selectedDay == nil || selectedDay == row["date"].text ? 1 : 0.3)
                }
            }.chartXSelection(value: $selectedDay)
            .chartXAxis { AxisMarks(values: rows.enumerated().filter { $0.offset % 7 == 0 }.map { $0.element["date"].text }) { value in
                AxisValueLabel { if let date = value.as(String.self) { Text(String(date.suffix(2))) } }
            } }
            .chartYAxis { AxisMarks(position: .trailing, values: .automatic(desiredCount: 3)) }
            .frame(height: 240)
        }
    }
    private func load() async {
        selectedDay = nil
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current
        guard let interval = calendar.dateInterval(of: .month, for: month) else { return }
        do {
            report = try await model.engine.request("/api/v1/spending/report", method: "POST", body: .strings([
                "startDate": ISO8601DateFormatter().string(from: interval.start),
                "endDate": ISO8601DateFormatter().string(from: interval.end.addingTimeInterval(-1))]))
            onReport(report)
            error = nil
        } catch { self.error = error.localizedDescription }
    }
}
struct SpendingSettingsView: View {
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @Environment(AppModel.self) private var model
    @State private var enabled = false
    @State private var selected: Set<String> = []
    @State private var error: String?
    var body: some View {
        Form {
            Toggle("Track spending", isOn: $enabled)
            Section("Accounts") {
                ForEach(model.accounts) { account in
                    Toggle(account["name"].text, isOn: Binding(get: { selected.contains(account.id) }, set: {
                        if $0 { selected.insert(account.id) } else { selected.remove(account.id) }
                    }))
                }
            }
            if let error { Text(error).foregroundStyle(.red) }
            Button("Save") { Task {
                do { _ = try await model.mutate("/api/v1/spending/settings", method: "PUT", body: .object([
                    "enabled": .bool(enabled), "accountIds": .array(selected.sorted().map(JSONValue.string))])) }
                catch { self.error = error.localizedDescription }
            } }
        }.scrollContentBackground(.hidden).background(palette.canvas).tint(palette.accent).leadingPageTitle("Spending accounts").task {
            do { let data = try await model.engine.request("/api/v1/spending/settings"); enabled = data["enabled"].flag; selected = Set(data["accountIds"].values.map(\.text)) }
            catch { self.error = error.localizedDescription }
        }
    }
}
