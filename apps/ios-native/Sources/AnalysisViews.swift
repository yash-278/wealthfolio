import SwiftUI
import Charts

struct InsightPalette {
    let scheme: ColorScheme
    var canvas: Color { scheme == .dark ? Color(red: 0.067, green: 0.106, blue: 0.125) : Color(red: 0.96, green: 0.975, blue: 0.98) }
    var surface: Color { scheme == .dark ? Color(red: 0.11, green: 0.165, blue: 0.188) : .white }
    var accent: Color { scheme == .dark ? Color(red: 0.62, green: 0.86, blue: 0.83) : Color(red: 0.16, green: 0.46, blue: 0.44) }
    var blue: Color { scheme == .dark ? Color(red: 0.55, green: 0.76, blue: 1) : Color(red: 0.16, green: 0.38, blue: 0.66) }
}

extension View {
    @ViewBuilder func leadingPageTitle(_ title: String, enabled: Bool = true) -> some View {
        if enabled { navigationTitle("").navigationBarTitleDisplayMode(.inline)
            .toolbarBackground(.hidden, for: .navigationBar)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Text(title).font(.headline).fixedSize(horizontal: true, vertical: false)
                        .accessibilityAddTraits(.isHeader)
                }.sharedBackgroundVisibility(.hidden)
            } } else { self }
    }
}

// Forms and lists keep system controls; only canvas, row surface and tint change.
private struct ThemedForm: ViewModifier {
    @Environment(\.colorScheme) private var scheme
    func body(content: Content) -> some View {
        let palette = InsightPalette(scheme: scheme)
        content.scrollContentBackground(.hidden).background(palette.canvas).tint(palette.accent)
    }
}
private struct SurfaceRows: ViewModifier {
    @Environment(\.colorScheme) private var scheme
    func body(content: Content) -> some View { content.listRowBackground(InsightPalette(scheme: scheme).surface) }
}
extension View {
    func themedForm() -> some View { modifier(ThemedForm()) }
    func surfaceRows() -> some View { modifier(SurfaceRows()) }
}

struct IconTile: View {
    let symbol: String
    var tint: Color
    var body: some View {
        Image(systemName: symbol).font(.body).foregroundStyle(tint).frame(width: 40, height: 40)
            .background(tint.opacity(0.1), in: .rect(cornerRadius: 12)).accessibilityHidden(true)
    }
}

struct PerformanceView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var data: JSONValue = .null
    @State private var days = 365
    @State private var error: String?
    @State private var loading = false
    @State private var selectedDate: Date?
    private struct ReturnPoint: Identifiable {
        let date: Date
        let value: Double
        var id: Date { date }
    }
    private var points: [ReturnPoint] {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current
        formatter.dateFormat = "yyyy-MM-dd"
        return data["series"].values.compactMap { point in
            guard let date = formatter.date(from: point["date"].text), let value = point["value"].decimal else { return nil }
            return ReturnPoint(date: date, value: NSDecimalNumber(decimal: value).doubleValue)
        }.sorted { $0.date < $1.date }
    }
    private var returnChart: some View {
        let series = points
        let selected = selectedDate.flatMap { date in series.min { abs($0.date.timeIntervalSince(date)) < abs($1.date.timeIntervalSince(date)) } }
        return VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text(selected?.date.formatted(date: .abbreviated, time: .omitted) ?? "Return over time")
                    .font(.caption).foregroundStyle(.secondary)
                Spacer()
                if let selected { Text(selected.value.formatted(.percent.precision(.fractionLength(2)))).font(.subheadline.weight(.semibold)).monospacedDigit() }
            }
            Chart(series) { point in
                AreaMark(x: .value("Date", point.date), yStart: .value("Baseline", 0), yEnd: .value("Return", point.value))
                    .foregroundStyle(LinearGradient(colors: [palette.accent.opacity(0.25), palette.accent.opacity(0.02)], startPoint: .top, endPoint: .bottom))
                LineMark(x: .value("Date", point.date), y: .value("Return", point.value))
                    .foregroundStyle(palette.accent)
                    .lineStyle(StrokeStyle(lineWidth: 2.5, lineCap: .round, lineJoin: .round))
                if point.id == (selected?.id ?? series.last?.id) {
                    PointMark(x: .value("Date", point.date), y: .value("Return", point.value))
                        .foregroundStyle(palette.accent).symbolSize(24)
                }
                if point.id == selected?.id {
                    RuleMark(x: .value("Date", point.date)).foregroundStyle(Color.secondary.opacity(0.4)).lineStyle(StrokeStyle(dash: [3, 4]))
                }
            }
            .chartXSelection(value: $selectedDate)
            .chartXAxis { AxisMarks(values: .automatic(desiredCount: 3)) { _ in AxisValueLabel(format: .dateTime.day().month(.abbreviated)) } }
            .chartYAxis { AxisMarks(position: .trailing, values: .automatic(desiredCount: 3)) { _ in
                AxisGridLine(stroke: StrokeStyle(dash: [2, 6])).foregroundStyle(Color.secondary.opacity(0.15))
                AxisValueLabel(format: FloatingPointFormatStyle<Double>.Percent())
            } }
            .frame(height: 280)
        }.padding(.vertical, 12)
    }
    private func percent(_ value: JSONValue) -> String {
        guard let amount = value.decimal else { return "—" }
        return amount.formatted(.percent.precision(.fractionLength(2)))
    }
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                Picker("Period", selection: $days) {
                    Text("1M").tag(30); Text("3M").tag(90); Text("1Y").tag(365); Text("All").tag(0)
                }.pickerStyle(.segmented)
                if loading { ProgressView("Calculating performance") }
                if let error { Text(error).foregroundStyle(.red) }
                if data != .null {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Time-weighted return").font(.subheadline).foregroundStyle(.secondary)
                        Text(percent(data["returns"]["twr"]))
                            .font(.system(.largeTitle, design: .rounded).weight(.semibold)).monospacedDigit()
                        Text(money(data["summary"]["amount"], currency: data["scope"]["currency"].text, hidden: model.hideBalances) + " gain")
                            .font(.subheadline).foregroundStyle(.secondary)
                    }
                    if !points.isEmpty { returnChart }
                    else { ContentUnavailableView("No return history", systemImage: "chart.xyaxis.line") }
                    if !model.hideBalances { attributionChart }
                    VStack(alignment: .leading, spacing: 16) {
                        Text("Risk").font(.headline)
                        LabeledContent("Volatility", value: percent(data["risk"]["volatility"]))
                        LabeledContent("Maximum drawdown", value: percent(data["risk"]["maxDrawdown"]))
                    }.font(.subheadline).padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                    DisclosureGroup("Return details") {
                        VStack(spacing: 16) {
                            LabeledContent("Annualized time-weighted", value: percent(data["returns"]["annualizedTwr"]))
                            LabeledContent("Money-weighted", value: percent(data["returns"]["irr"]))
                            LabeledContent("Annualized money-weighted", value: percent(data["returns"]["annualizedIrr"]))
                            LabeledContent("Value return", value: percent(data["returns"]["valueReturn"]))
                            ForEach(["contributions", "distributions", "income", "realizedPnl", "unrealizedPnlChange", "fxEffect", "fees", "taxes", "residual"], id: \.self) { key in
                                LabeledContent(attributionLabel(key), value: money(data["attribution"][key], currency: data["scope"]["currency"].text, hidden: model.hideBalances))
                            }
                        }.font(.subheadline).padding(.top, 16)
                    }.padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                    ForEach(Array((data["dataQuality"]["warnings"].values + data["dataQuality"]["notApplicableReasons"].values).enumerated()), id: \.offset) { _, warning in
                        Label(warning.text, systemImage: "info.circle").font(.footnote).foregroundStyle(.secondary)
                    }
                }
            }.padding(24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).tint(palette.accent)
        .leadingPageTitle("Performance").task(id: "\(days)-\(model.revision)") { await load() }.refreshable { await load() }
    }
    @State private var selectedEffect: String?
    private var attributionChart: some View {
        let keys = ["income", "realizedPnl", "unrealizedPnlChange", "fxEffect", "fees", "taxes"]
        let present = keys.filter { data["attribution"][$0].decimal != nil }
        return VStack(alignment: .leading, spacing: 16) {
            Text("Gain and loss components").font(.headline)
            if let key = selectedEffect {
                LabeledContent(attributionLabel(key), value: money(.number(effectAmount(key)), currency: data["scope"]["currency"].text))
                    .font(.subheadline).monospacedDigit()
            }
            Chart(present, id: \.self) { key in
                let value = NSDecimalNumber(decimal: effectAmount(key)).doubleValue
                BarMark(x: .value("Amount", value), y: .value("Component", key))
                    .foregroundStyle(value < 0 ? Color.orange : palette.accent)
                    .opacity(selectedEffect == nil || selectedEffect == key ? 1 : 0.35)
                    .cornerRadius(4)
                    .accessibilityLabel(attributionLabel(key))
                    .accessibilityValue(money(.number(effectAmount(key)), currency: data["scope"]["currency"].text))
            }
            .chartYSelection(value: $selectedEffect)
            .chartYAxis { AxisMarks { value in AxisValueLabel { if let key = value.as(String.self) { Text(attributionLabel(key)).font(.caption) } } } }
            .chartXAxis { AxisMarks(values: .automatic(desiredCount: 3)) }
            .frame(height: 220)
            Text(data["scope"]["currency"].text).font(.caption).foregroundStyle(.secondary)
        }.padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
    }
    private func effectAmount(_ key: String) -> Decimal {
        let amount = data["attribution"][key].decimal ?? 0
        return key == "fees" || key == "taxes" ? -amount : amount
    }
    private func attributionLabel(_ key: String) -> String {
        ["realizedPnl": "Realized gain", "unrealizedPnlChange": "Unrealized gain change", "fxEffect": "Currency effect"][key] ?? key.capitalized
    }
    private func load() async {
        selectedDate = nil
        loading = true; error = nil; defer { loading = false }
        var body: [String: JSONValue] = ["itemType": .string("account"), "itemId": .string("TOTAL"), "filter": .strings(["type": "all"])]
        if days > 0 {
            var calendar = Calendar(identifier: .gregorian)
            calendar.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current
            let formatter = DateFormatter(); formatter.calendar = calendar; formatter.locale = Locale(identifier: "en_US_POSIX"); formatter.timeZone = calendar.timeZone; formatter.dateFormat = "yyyy-MM-dd"
            body["startDate"] = .string(formatter.string(from: calendar.date(byAdding: .day, value: -days, to: .now) ?? .now))
        }
        do {
            let result = try await model.engine.request("/api/v1/performance/history", method: "POST", body: .object(body))
            try Task.checkCancellation(); data = result
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
}

struct AllocationsView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var loading = true
    @State private var data: JSONValue = .null
    @State private var error: String?
    @State private var taxonomyIndex = 0
    @State private var selectedIndex: Int?
    @State private var selectedAngle: Double?
    private var taxonomies: [JSONValue] {
        (["assetClasses", "sectors", "regions", "riskCategory", "securityTypes"].map { data[$0] } + data["customGroups"].values)
            .filter { !$0["categories"].values.isEmpty }
    }
    private var categories: [JSONValue] { taxonomies.indices.contains(taxonomyIndex) ? taxonomies[taxonomyIndex]["categories"].values : [] }
    private func color(_ index: Int) -> Color {
        [palette.accent, palette.blue, Color.orange, Color.purple, Color.pink, Color.gray][index % 6]
    }
    private var selected: JSONValue? { selectedIndex.flatMap { categories.indices.contains($0) ? categories[$0] : nil } }
    private var canUseRing: Bool { !categories.isEmpty && categories.allSatisfy { ($0["value"].decimal ?? 0) >= 0 } && categories.contains { ($0["value"].decimal ?? 0) > 0 } }
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                if loading { ProgressView("Loading allocation") }
                if let error { Text(error).foregroundStyle(.red) }
                if !taxonomies.isEmpty {
                    Picker("Classification", selection: $taxonomyIndex) {
                        ForEach(taxonomies.indices, id: \.self) { index in Text(taxonomies[index]["taxonomyName"].text).tag(index) }
                    }.pickerStyle(.menu).buttonStyle(.glass)
                    if canUseRing {
                        Chart(Array(categories.enumerated()), id: \.offset) { index, category in
                            SectorMark(angle: .value("Value", NSDecimalNumber(decimal: category["value"].decimal ?? 0).doubleValue), innerRadius: .ratio(0.7), outerRadius: .ratio(selectedIndex == index ? 1 : 0.94), angularInset: 2)
                                .cornerRadius(5).foregroundStyle(color(index))
                                .opacity(selectedIndex == nil || selectedIndex == index ? 1 : 0.3)
                                .accessibilityLabel(category["categoryName"].text)
                                .accessibilityValue(share(category))
                        }.chartAngleSelection(value: $selectedAngle).frame(height: 280)
                        .chartBackground { _ in
                            VStack(spacing: 6) {
                                Text(selected.map(share) ?? "Allocation")
                                    .font(.system(.largeTitle, design: .rounded).weight(.semibold)).monospacedDigit()
                                Text(selected?["categoryName"].text ?? "Portfolio").font(.caption).foregroundStyle(.secondary).multilineTextAlignment(.center)
                            }.frame(maxWidth: 150).allowsHitTesting(false)
                        }
                    }
                    if let selected {
                        LabeledContent(selected["categoryName"].text, value: money(selected["value"], currency: model.currency, hidden: model.hideBalances))
                            .font(.headline).monospacedDigit().padding(20).background(palette.surface, in: .rect(cornerRadius: 20))
                    }
                    VStack(spacing: 4) {
                        ForEach(Array(categories.enumerated()), id: \.offset) { index, category in
                            Button { selectedIndex = selectedIndex == index ? nil : index } label: {
                                HStack(spacing: 12) {
                                    Circle().fill(color(index)).frame(width: 10, height: 10).accessibilityHidden(true)
                                    Text(category["categoryName"].text).font(.subheadline.weight(.medium)).frame(maxWidth: .infinity, alignment: .leading)
                                    VStack(alignment: .trailing, spacing: 5) {
                                        Text(share(category)).font(.subheadline.weight(.semibold))
                                        Text(money(category["value"], currency: model.currency, hidden: model.hideBalances)).font(.caption).foregroundStyle(.secondary)
                                    }.monospacedDigit()
                                }.padding(16).background(selectedIndex == index ? palette.surface : .clear, in: .rect(cornerRadius: 16)).contentShape(.rect)
                            }.buttonStyle(.plain).accessibilityAddTraits(selectedIndex == index ? [.isSelected] : [])
                        }
                    }
                } else if !loading && error == nil { ContentUnavailableView("No allocation yet", systemImage: "chart.pie") }
            }.padding(24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).tint(palette.accent).leadingPageTitle("Allocation")
        .onChange(of: taxonomyIndex) { _, _ in selectedIndex = nil; selectedAngle = nil }
        .onChange(of: selectedAngle) { _, angle in
            guard let angle else { return }
            var sum = 0.0
            selectedIndex = categories.indices.first { index in
                sum += max(0, NSDecimalNumber(decimal: categories[index]["value"].decimal ?? 0).doubleValue)
                return angle < sum
            }
        }
        .task(id: model.revision) { await load() }.refreshable { await load() }
    }
    private func share(_ category: JSONValue) -> String {
        guard let percentage = category["percentage"].decimal else { return "—" }
        return (percentage / 100).formatted(.percent.precision(.fractionLength(1)))
    }
    private func load() async {
        loading = true; defer { loading = false }
        do {
            data = try await model.engine.request("/api/v1/allocations/query", method: "POST", body: .object(["filter": .strings(["type": "all"])]))
            taxonomyIndex = min(taxonomyIndex, max(0, taxonomies.count - 1)); selectedIndex = nil; selectedAngle = nil; error = nil
        } catch { self.error = error.localizedDescription }
    }
}

struct ExportView: View {
    @Environment(AppModel.self) private var model
    @State private var kind = "activities"
    @State private var file: URL?
    @State private var working = false
    @State private var error: String?
    var body: some View {
        Form {
            Section {
                Picker("Data", selection: $kind) {
                    Text("Transactions").tag("activities"); Text("Accounts").tag("accounts"); Text("Holdings").tag("holdings"); Text("Goals").tag("goals"); Text("Portfolio history").tag("portfolio-history")
                }.onChange(of: kind) { _, _ in file = nil }
            } footer: { Text("The file is prepared on this device and is only shared when you choose a destination.") }.surfaceRows()
            Section {
                Button { Task { await prepare() } } label: {
                    HStack { Label("Prepare CSV", systemImage: "doc.badge.gearshape"); Spacer(); if working { ProgressView() } }
                }.disabled(working)
                if let file { ShareLink(item: file) { Label("Save or share CSV", systemImage: "square.and.arrow.up") } }
            }.surfaceRows()
            if let error { Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }.surfaceRows() }
        }.themedForm().leadingPageTitle("Export")
    }
    private func prepare() async {
        working = true; file = nil; error = nil; defer { working = false }
        do {
            let result = try await model.engine.request("/api/v1/utilities/export/\(kind)/csv")
            guard !result.text.isEmpty else { error = "There is no data to export."; return }
            let directory = FileManager.default.temporaryDirectory.appending(path: "Exports")
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let url = directory.appending(path: "wealthfolio-\(kind).csv")
            try Data(result.text.utf8).write(to: url, options: [.atomic, .completeFileProtection]); file = url
        } catch { self.error = error.localizedDescription }
    }
}
