import SwiftUI
import Charts

struct HoldingsView: View {
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @Environment(AppModel.self) private var model
    var accountID: String?
    @State private var rows: [Record] = []
    @State private var error: String?
    @State private var query = ""
    var body: some View {
        List {
            if let error { Text(error).foregroundStyle(.red) }
            ForEach(rows.filter { query.isEmpty || $0["instrument"]["symbol"].text.localizedCaseInsensitiveContains(query) || $0["instrument"]["name"].text.localizedCaseInsensitiveContains(query) }) { holding in
                NavigationLink { HoldingDetailView(holding: holding) } label: {
                    HStack(spacing: 14) {
                        Image(systemName: holding["instrument"]["symbol"].text.isEmpty ? "banknote" : "chart.line.uptrend.xyaxis")
                            .foregroundStyle(palette.accent).frame(width: 42, height: 42)
                            .background(palette.accent.opacity(0.1), in: .rect(cornerRadius: 12)).accessibilityHidden(true)
                        VStack(alignment: .leading, spacing: 5) {
                            Text(holding["instrument"]["symbol"].text.isEmpty ? "Cash" : holding["instrument"]["symbol"].text).font(.body.weight(.medium))
                            Text(holding["instrument"]["name"].text).font(.caption).foregroundStyle(.secondary)
                        }; Spacer()
                        Text(money(holding["marketValue"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances)).font(.subheadline.weight(.semibold)).monospacedDigit()
                    }.padding(.vertical, 10)
                }.listRowBackground(Color.clear).listRowSeparator(.hidden)
            }
            if rows.isEmpty && error == nil { ContentUnavailableView("No holdings", systemImage: "briefcase") }
        }.listStyle(.plain).scrollContentBackground(.hidden).background(palette.canvas).tint(palette.accent).leadingPageTitle("Holdings").searchable(text: $query)
        .task(id: model.revision) { await load() }.refreshable { await load() }
    }
    private func load() async {
        do {
            var filter: JSONValue = .object(["type": .string("all")])
            if let accountID { filter = .strings(["type": "account", "accountId": accountID]) }
            rows = try await model.engine.request("/api/v1/holdings/list/query", method: "POST", body: .object(["filter": filter])).values.map(Record.init)
        } catch { self.error = error.localizedDescription }
    }
}
struct HoldingDetailView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    let holding: Record
    var body: some View {
        List {
            Section {
            LabeledContent("Quantity", value: holding["quantity"].text)
            LabeledContent("Price", value: money(holding["price"], currency: holding["localCurrency"].text, hidden: model.hideBalances))
            LabeledContent("Market value", value: money(holding["marketValue"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances))
            LabeledContent("Cost basis", value: money(holding["costBasis"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances))
            LabeledContent("Unrealized gain", value: money(holding["unrealizedGain"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances))
            LabeledContent("Realized gain", value: money(holding["realizedGain"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances))
            LabeledContent("Income", value: money(holding["income"]["base"], currency: holding["baseCurrency"].text, hidden: model.hideBalances))
            LabeledContent("As of", value: holding["asOfDate"].text)
            }.listRowBackground(palette.surface)
        }.listStyle(.insetGrouped).scrollContentBackground(.hidden).background(palette.canvas).tint(palette.accent).leadingPageTitle(holding["instrument"]["symbol"].text.isEmpty ? "Cash" : holding["instrument"]["symbol"].text)
    }
}
struct NetWorthView: View {
    var embedded = false
    var dashboardHeader: AnyView? = nil
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @Environment(AppModel.self) private var model
    @State private var data: JSONValue = .null
    @State private var error: String?
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                dashboardHeader
                if let error { Text(error).foregroundStyle(.red) }
                if !embedded { Text(money(data["netWorth"], currency: data["currency"].text, hidden: model.hideBalances))
                    .font(.system(.largeTitle, design: .rounded).weight(.semibold)).monospacedDigit().minimumScaleFactor(0.6).lineLimit(1) }
                if !model.hideBalances && data != .null {
                    Chart(["assets", "liabilities"], id: \.self) { key in
                        if let amount = data[key]["total"].decimal {
                            BarMark(x: .value("Value", NSDecimalNumber(decimal: amount).doubleValue), y: .value("Type", key.capitalized))
                                .foregroundStyle(key == "assets" ? palette.accent : Color.orange).cornerRadius(6)
                        }
                    }.chartXAxis(.hidden).frame(height: 130).accessibilityLabel("Assets and liabilities")
                }
                VStack(spacing: 20) {
                    LabeledContent("Assets", value: money(data["assets"]["total"], currency: data["currency"].text, hidden: model.hideBalances))
                    LabeledContent("Liabilities", value: money(data["liabilities"]["total"], currency: data["currency"].text, hidden: model.hideBalances))
                }.font(.body.weight(.medium)).monospacedDigit().padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                if !data["date"].text.isEmpty { Text("As of " + data["date"].text).font(.caption).foregroundStyle(.secondary) }
            }.padding(24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).leadingPageTitle("Net worth", enabled: !embedded).task(id: model.revision) {
            do { data = try await model.engine.request("/api/v1/net-worth"); error = nil } catch { self.error = error.localizedDescription }
        }
    }
}

struct InsightsView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    private var currencies: [JSONValue] { model.valuation["summary"]["currencySplit"].values }
    private var largestCurrency: Double {
        currencies.compactMap { $0["valueBase"].decimal.map { abs(NSDecimalNumber(decimal: $0).doubleValue) } }.max() ?? 0
    }
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                VStack(spacing: 12) {
                    NavigationLink { PerformanceView() } label: {
                        reportCard("Performance", symbol: "chart.xyaxis.line", tint: palette.accent)
                    }
                    NavigationLink { AllocationsView() } label: {
                        reportCard("Allocation", symbol: "chart.pie", tint: palette.blue)
                    }
                }.buttonStyle(.plain)
                if !currencies.isEmpty {
                    VStack(alignment: .leading, spacing: 18) {
                        HStack {
                            Text("Currency exposure").font(.title3.weight(.semibold))
                            Spacer()
                            Text(model.currency).font(.caption).foregroundStyle(.secondary)
                        }
                        ForEach(Array(currencies.enumerated()), id: \.offset) { index, row in
                            VStack(alignment: .leading, spacing: 9) {
                                LabeledContent(row["currency"].text, value: money(row["valueBase"], currency: model.currency, hidden: model.hideBalances))
                                    .font(.subheadline.weight(.medium)).monospacedDigit()
                                if !model.hideBalances, let value = row["valueBase"].decimal, largestCurrency > 0 {
                                    GeometryReader { geometry in
                                        Capsule().fill(palette.accent.opacity(0.09))
                                        Capsule().fill(index.isMultiple(of: 2) ? palette.accent : palette.blue)
                                            .frame(width: geometry.size.width * min(1, abs(NSDecimalNumber(decimal: value).doubleValue) / largestCurrency))
                                    }.frame(height: 6).accessibilityHidden(true)
                                }
                            }
                        }
                        Text("Values converted to \(model.currency).")
                            .font(.caption).foregroundStyle(.secondary)
                    }.padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                }
                VStack(alignment: .leading, spacing: 8) {
                    Text("Explore more").font(.title3.weight(.semibold)).padding(.bottom, 4)
                    NavigationLink { HoldingsView() } label: { reportRow("Holdings", symbol: "square.stack.3d.up") }
                    NavigationLink { NetWorthView() } label: { reportRow("Net worth", symbol: "wallet.bifold") }
                    NavigationLink { GoalsView() } label: { reportRow("Goals", symbol: "target") }
                    NavigationLink { SpendingView() } label: { reportRow("Spending", symbol: "creditcard") }
                }.buttonStyle(.plain)
            }.padding(.horizontal, 20).padding(.vertical, 24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).leadingPageTitle("Insights")
        .toolbar { ToolbarItem(placement: .topBarTrailing) {
            Button(model.hideBalances ? "Show balances" : "Hide balances", systemImage: model.hideBalances ? "eye.slash" : "eye") { model.hideBalances.toggle() }
        } }
    }
    private func reportCard(_ title: String, symbol: String, tint: Color) -> some View {
        HStack(spacing: 16) {
            Image(systemName: symbol).font(.title2).foregroundStyle(tint)
                .frame(width: 48, height: 48).background(tint.opacity(0.1), in: .rect(cornerRadius: 14)).accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 6) {
                Text(title).font(.headline).foregroundStyle(.primary)
            }.frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(.tertiary).accessibilityHidden(true)
        }.padding(20).background(palette.surface, in: .rect(cornerRadius: 22)).contentShape(.rect)
    }
    private func reportRow(_ title: String, symbol: String) -> some View {
        HStack(spacing: 14) {
            Image(systemName: symbol).font(.body).foregroundStyle(palette.accent).frame(width: 28).accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 4) {
                Text(title).font(.body.weight(.medium)).foregroundStyle(.primary)
            }.frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right").font(.caption).foregroundStyle(.tertiary).accessibilityHidden(true)
        }.padding(.vertical, 12).contentShape(.rect)
    }
}
struct GoalsView: View {
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @Environment(AppModel.self) private var model
    @State private var rows: [Record] = []
    @State private var adding = false
    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                ForEach(rows) { goal in
                    VStack(alignment: .leading, spacing: 16) {
                        HStack(spacing: 16) {
                            if let progress = goal["summaryProgress"].decimal {
                                ZStack {
                                    Circle().stroke(palette.accent.opacity(0.12), lineWidth: 6)
                                    Circle().trim(from: 0, to: min(1, max(0, NSDecimalNumber(decimal: progress).doubleValue)))
                                        .stroke(palette.accent, style: StrokeStyle(lineWidth: 6, lineCap: .round)).rotationEffect(.degrees(-90))
                                    Image(systemName: "target").foregroundStyle(palette.accent)
                                }.frame(width: 48, height: 48).accessibilityHidden(true)
                            }
                            Text(goal["title"].text).font(.headline)
                        }
                        Text(money(goal["summaryCurrentValue"], currency: goal["currency"].text, hidden: model.hideBalances))
                            .font(.title2.weight(.semibold)).monospacedDigit()
                        HStack {
                            Text("of " + money(goal["targetAmount"], currency: goal["currency"].text, hidden: model.hideBalances))
                            Spacer()
                            if let progress = goal["summaryProgress"].decimal { Text(progress.formatted(.percent.precision(.fractionLength(0)))) }
                        }.font(.subheadline).foregroundStyle(.secondary)
                    }.padding(20).frame(maxWidth: .infinity, alignment: .leading).background(palette.surface, in: .rect(cornerRadius: 22))
                }
                if rows.isEmpty { ContentUnavailableView("No goals", systemImage: "target").frame(maxWidth: .infinity) }
            }.padding(24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).tint(palette.accent).leadingPageTitle("Goals")
        .toolbar { ToolbarItem(placement: .topBarTrailing) { Button("Add goal", systemImage: "plus") { adding = true } } }
        .task(id: model.revision) { do { rows = try await model.engine.request("/api/v1/goals").values.map(Record.init) } catch { model.error = error.localizedDescription } }
        .sheet(isPresented: $adding) { NavigationStack { GoalEditor() } }
    }
}
struct GoalEditor: View {
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    @State private var title = ""
    @State private var amount = ""
    @State private var date = Date()
    @State private var error: String?
    var body: some View {
        Form {
            TextField("Goal name", text: $title)
            TextField("Target amount", text: $amount).keyboardType(.decimalPad)
            DatePicker("Target date", selection: $date, displayedComponents: .date)
            if let error { Text(error).foregroundStyle(.red) }
        }.scrollContentBackground(.hidden).background(palette.canvas).tint(palette.accent).navigationTitle("New goal").navigationBarTitleDisplayMode(.inline).toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) { Button("Save") { Task {
                guard let number = Decimal(string: amount) else { error = "Enter a valid target amount."; return }
                do {
                    _ = try await model.mutate("/api/v1/goals", body: .object(["title": .string(title), "goalType": .string("custom_save_up"), "targetAmount": .number(number), "currency": .string(model.currency), "targetDate": .string(String(ISO8601DateFormatter().string(from: date).prefix(10)))])); dismiss()
                } catch { self.error = error.localizedDescription }
            } }.disabled(title.isEmpty || amount.isEmpty) }
        }
    }
}
