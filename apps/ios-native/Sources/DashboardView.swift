import SwiftUI
import Charts
import UIKit


struct DashboardPoint: Identifiable {
    let date: Date
    let value: Decimal
    var id: Date { date }
    var plotted: Double { NSDecimalNumber(decimal: value).doubleValue }
}

// UIKit owns the transition endpoints. Core Animation interpolates the layers;
// SwiftUI only updates this view when the selection or displayed data changes.
private struct DashboardMetricCards: UIViewRepresentable {
    var values: [String]
    var active: Int
    var dark: Bool
    var fontSize: CGFloat
    var stacked: Bool
    var reduceMotion: Bool
    var onSelect: (Int) -> Void

    var height: CGFloat {
        let main = 56 + fontSize * 1.25
        let small = 56 + fontSize * 1.25 * 0.43
        return main + 12 + (stacked ? small * 2 + 12 : small)
    }
    func makeUIView(context: Context) -> DashboardCardsView { DashboardCardsView() }
    func updateUIView(_ view: DashboardCardsView, context: Context) {
        view.update(self)
    }
}

private final class DashboardMetricControl: UIControl {
    let glass = UIVisualEffectView(effect: UIGlassEffect(style: .regular))
    let title = UILabel()
    let value = UILabel()
    let arrow = UIImageView(image: UIImage(systemName: "arrow.up.left"))
    override init(frame: CGRect) {
        super.init(frame: frame)
        // Content belongs inside the effect view so glass never samples the labels.
        glass.cornerConfiguration = .corners(radius: .fixed(24))
        glass.isUserInteractionEnabled = false
        addSubview(glass)
        title.font = .preferredFont(forTextStyle: .caption1)
        value.numberOfLines = 1
        value.adjustsFontSizeToFitWidth = true
        value.minimumScaleFactor = 0.65
        [title, value, arrow].forEach { $0.isUserInteractionEnabled = false; glass.contentView.addSubview($0) }
        isAccessibilityElement = true
    }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
}

private final class DashboardCardsView: UIView {
    private let cards = (0..<3).map { _ in DashboardMetricControl() }
    private let glassContainer: UIVisualEffectView = {
        let effect = UIGlassContainerEffect()
        effect.spacing = 8
        return UIVisualEffectView(effect: effect)
    }()
    private var configuration: DashboardMetricCards?
    private var laidOutSize = CGSize.zero
    private let names = ["Investments", "Net worth", "Spending"]

    override init(frame: CGRect) {
        super.init(frame: frame)
        addSubview(glassContainer)
        for (index, card) in cards.enumerated() {
            card.tag = index
            card.addTarget(self, action: #selector(selectCard(_:)), for: .touchUpInside)
            glassContainer.contentView.addSubview(card)
        }
    }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    @objc private func selectCard(_ sender: UIControl) { configuration?.onSelect(sender.tag) }

    func update(_ next: DashboardMetricCards) {
        let previous = configuration
        configuration = next
        for (index, card) in cards.enumerated() {
            card.title.text = names[index]
            card.title.textColor = next.dark ? UIColor(white: 0.68, alpha: 1) : .secondaryLabel
            card.value.text = next.values[index]
            let descriptor = UIFont.monospacedDigitSystemFont(ofSize: next.fontSize, weight: .semibold).fontDescriptor
            card.value.font = UIFont(descriptor: descriptor.withDesign(.rounded) ?? descriptor, size: next.fontSize)
            card.value.textColor = next.dark ? .white : .black
            card.arrow.tintColor = next.dark ? UIColor(red: 0.62, green: 0.86, blue: 0.83, alpha: 1) : UIColor(red: 0.16, green: 0.46, blue: 0.44, alpha: 1)
            card.accessibilityLabel = "\(names[index]), \(next.values[index])"
            card.accessibilityTraits = index == next.active ? [.button, .selected] : [.button]
            // Keep effects alive across selections; only reconfigure on theme changes.
            if previous == nil || previous?.dark != next.dark {
                let effect = UIGlassEffect(style: .regular)
                effect.tintColor = card.arrow.tintColor.withAlphaComponent(0.04)
                card.glass.effect = effect
            }
        }
        guard bounds.width > 0 else { return }
        let animate = previous != nil && previous?.active != next.active && !next.reduceMotion
        if animate {
            UIView.animate(withDuration: 0.58, delay: 0, usingSpringWithDamping: 0.86, initialSpringVelocity: 0,
                           options: [.beginFromCurrentState, .allowUserInteraction]) { self.placeCards() }
        } else {
            UIView.performWithoutAnimation { self.placeCards() }
        }
    }
    override func layoutSubviews() {
        super.layoutSubviews()
        guard bounds.size != laidOutSize else { return }
        laidOutSize = bounds.size
        glassContainer.frame = bounds
        UIView.performWithoutAnimation { placeCards() }
    }
    private func placeCards() {
        guard let config = configuration else { return }
        let width = bounds.width
        let mainHeight = 56 + config.fontSize * 1.25
        let smallHeight = 56 + config.fontSize * 1.25 * 0.43
        let smallWidth = config.stacked ? width : (width - 12) / 2
        let others = cards.indices.filter { $0 != config.active }
        for (index, card) in cards.enumerated() {
            let prominent = index == config.active
            let slot = others.firstIndex(of: index) ?? 0
            let x = prominent || config.stacked ? 0 : CGFloat(slot) * (smallWidth + 12)
            let y = prominent ? 0 : mainHeight + 12 + (config.stacked ? CGFloat(slot) * (smallHeight + 12) : 0)
            let cardWidth = prominent ? width : smallWidth
            card.frame = CGRect(x: x, y: y, width: cardWidth, height: prominent ? mainHeight : smallHeight)
            card.glass.frame = card.bounds
            card.title.frame = CGRect(x: 16, y: 16, width: max(1, cardWidth - 50), height: 16)
            card.arrow.frame = CGRect(x: cardWidth - 30, y: 18, width: 12, height: 12)
            card.arrow.alpha = prominent ? 0 : 1
            // Keep the text backing at one font size; animate its transform only.
            let scale: CGFloat = prominent ? 1 : min(0.43, (cardWidth - 32) / max(1, width - 32))
            let textSize = CGSize(width: max(1, width - 32), height: config.fontSize * 1.25)
            card.value.bounds = CGRect(origin: .zero, size: textSize)
            card.value.center = CGPoint(x: 16 + textSize.width * scale / 2, y: 40 + textSize.height * scale / 2)
            card.value.transform = CGAffineTransform(scaleX: scale, y: scale)
        }
    }
}

private struct DashboardSectionSwitcher: UIViewRepresentable {
    var sections: [String]
    var selection: String
    var dark: Bool
    var reduceMotion: Bool
    var onSelect: (String) -> Void

    func makeUIView(context: Context) -> DashboardSectionBackdrop { DashboardSectionBackdrop() }
    func updateUIView(_ view: DashboardSectionBackdrop, context: Context) { view.control.update(self) }
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: DashboardSectionBackdrop, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 354, height: uiView.control.intrinsicContentSize.height)
    }
}

// Use native Liquid Glass within the track's exact footprint.
// Keep the system segmented control inside its content layer.
private final class DashboardSectionBackdrop: UIView {
    let control = DashboardSectionControl(frame: .zero)
    private let backdrop = UIVisualEffectView(effect: UIGlassEffect(style: .regular))
    init() {
        super.init(frame: .zero)
        addSubview(backdrop)
        backdrop.contentView.addSubview(control)
    }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    override func layoutSubviews() {
        super.layoutSubviews()
        backdrop.frame = bounds
        backdrop.cornerConfiguration = .corners(radius: .fixed(bounds.height / 2))
        control.frame = backdrop.contentView.bounds
    }
}

// Leave glass rendering and touch tracking to the system segmented control.
// The page binding changes only when tracking finishes.
private final class DashboardSectionControl: UISegmentedControl {
    private var configuration: DashboardSectionSwitcher?
    private var selectionBeforeTracking = UISegmentedControl.noSegment
    private var trackingTouch = false

    override init(frame: CGRect) {
        super.init(frame: frame)
        accessibilityIdentifier = "dashboard.sectionSwitcher"
        addTarget(self, action: #selector(selectionChanged), for: .valueChanged)
    }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    func update(_ next: DashboardSectionSwitcher) {
        let previous = configuration
        configuration = next
        if previous?.sections != next.sections {
            removeAllSegments()
            for (index, title) in next.sections.enumerated() {
                insertSegment(withTitle: title, at: index, animated: false)
            }
        }
        // Avoid resetting the system's tentative selection while the finger moves.
        if !trackingTouch {
            selectedSegmentIndex = next.sections.firstIndex(of: next.selection) ?? 0
        }
    }
    override func beginTracking(_ touch: UITouch, with event: UIEvent?) -> Bool {
        selectionBeforeTracking = selectedSegmentIndex
        trackingTouch = true
        let accepted = super.beginTracking(touch, with: event)
        trackingTouch = accepted
        return accepted
    }
    override func endTracking(_ touch: UITouch?, with event: UIEvent?) {
        super.endTracking(touch, with: event)
        trackingTouch = false
        commitSelection()
    }
    override func cancelTracking(with event: UIEvent?) {
        super.cancelTracking(with: event)
        selectedSegmentIndex = selectionBeforeTracking
        trackingTouch = false
    }
    @objc private func selectionChanged() {
        // VoiceOver and keyboard selection do not start touch tracking.
        if !trackingTouch { commitSelection() }
    }
    private func commitSelection() {
        guard let config = configuration, config.sections.indices.contains(selectedSegmentIndex) else { return }
        let selected = config.sections[selectedSegmentIndex]
        guard selected != config.selection else { return }
        configuration?.selection = selected
        config.onSelect(selected)
    }
}

struct DashboardView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    @Environment(\.dynamicTypeSize) private var textSize
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @ScaledMetric(relativeTo: .largeTitle) private var balanceSize = 42
    @ScaledMetric(relativeTo: .subheadline) private var circleSize = 44
    @AppStorage("native.dashboard.section") private var section = "Investments"
    @State private var netWorth: JSONValue = .null
    @State private var spending: JSONValue = .null
    @State private var spendingEnabled = false
    @State private var overviewError: String?
    @State private var spendingSetup = false
    @State private var adding = false
    @State private var quickAdd = false
    @State private var period = 90
    @State private var selectedDate: Date?
    @Namespace private var periodGlass
    private var accent: Color { scheme == .dark ? Color(red: 0.62, green: 0.86, blue: 0.83) : Color(red: 0.16, green: 0.46, blue: 0.44) }
    private var canvas: Color { scheme == .dark ? Color(red: 0.067, green: 0.106, blue: 0.125) : Color(red: 0.96, green: 0.975, blue: 0.98) }
    private var points: [DashboardPoint] {
        guard period != 0 else { return model.dashboardPoints }
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current
        let cutoff = calendar.date(byAdding: .day, value: -period, to: .now) ?? .now
        return model.dashboardPoints.filter { $0.date >= cutoff }
    }
    private var selection: DashboardPoint? {
        guard let selectedDate else { return nil }
        return points.min { abs($0.date.timeIntervalSince(selectedDate)) < abs($1.date.timeIntervalSince(selectedDate)) }
    }
    private var displayed: JSONValue { selection.map { .number($0.value) } ?? model.valuation["summary"]["totalValueBase"] }
    private var change: Decimal? {
        guard let first = points.first, points.count > 1, let current = displayed.decimal else { return nil }
        return current - first.value
    }
    private var range: ClosedRange<Double> {
        let values = points.map(\.plotted)
        let low = values.min() ?? 0, high = values.max() ?? 1
        let padding = max((high-low)*0.15, max(abs(high)*0.01, 1))
        return (low-padding)...(high+padding)
    }
    var body: some View {
        GeometryReader { viewport in
            ZStack(alignment: .top) {
                TabView(selection: $section) {
                    investmentsPage
                        .safeAreaPadding(.top, max(48, circleSize + 4) + 8)
                        .frame(width: viewport.size.width, height: viewport.size.height, alignment: .top)
                        .scrollClipDisabled()
                        .scrollEdgeEffectStyle(.soft, for: .top)
                        .tag("Investments")
                    NetWorthView(embedded: true, dashboardHeader: AnyView(valueHeader))
                        .safeAreaPadding(.top, max(48, circleSize + 4) + 8)
                        .frame(width: viewport.size.width, height: viewport.size.height, alignment: .top)
                        .scrollClipDisabled()
                        .scrollEdgeEffectStyle(.soft, for: .top)
                        .tag("Net worth")
                    if spendingEnabled {
                        SpendingView(embedded: true, dashboardHeader: AnyView(valueHeader), onReport: { spending = $0 })
                            .safeAreaPadding(.top, max(48, circleSize + 4) + 8)
                        .frame(width: viewport.size.width, height: viewport.size.height, alignment: .top)
                            .scrollClipDisabled()
                            .scrollEdgeEffectStyle(.soft, for: .top)
                            .tag("Spending")
                    }
                }.tabViewStyle(.page(indexDisplayMode: .never))
                    // Scroll behind the section switcher, but keep the viewport
                    // below the navigation title bar.
                    .contentMargins(.bottom, viewport.safeAreaInsets.bottom, for: .scrollContent)
                    // Keep the page viewport anchored below the navigation title.
                    .frame(width: viewport.size.width,
                           height: viewport.size.height + viewport.safeAreaInsets.bottom,
                           alignment: .top)
                    .clipped()
                DashboardSectionSwitcher(
                    sections: spendingEnabled ? ["Investments", "Net worth", "Spending"] : ["Investments", "Net worth"],
                    selection: section, dark: scheme == .dark, reduceMotion: reduceMotion
                ) { section = $0 }
                    .frame(height: max(48, circleSize + 4))
                    .padding(.horizontal, 24).padding(.top, 8)
            }
        }.background(canvas).tint(accent)
        .leadingPageTitle("Dashboard")
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button(model.hideBalances ? "Show balances" : "Hide balances", systemImage: model.hideBalances ? "eye.slash" : "eye") { model.hideBalances.toggle(); selectedDate = nil }
            }
            ToolbarItem(placement: .topBarTrailing) { Button("Quick Add", systemImage: "plus") { quickAdd = true } }
            ToolbarItem(placement: .topBarTrailing) {
                Menu("Actions", systemImage: "ellipsis") {
                    Button("Add account", systemImage: "plus") { adding = true }
                    Button("Sync now", systemImage: "arrow.triangle.2.circlepath") { Task { await model.synchronize() } }
                    Button("Update prices", systemImage: "arrow.clockwise") { Task {
                        do { _ = try await model.mutate("/api/v1/portfolio/update") } catch { model.error = error.localizedDescription }
                    } }
                    Button("Spending settings", systemImage: "slider.horizontal.3") { spendingSetup = true }
                }
            }
        }
        .onChange(of: section) { _, _ in selectedDate = nil }
        .task(id: model.revision) { await loadOverview() }
        .sheet(isPresented: $adding) { NavigationStack { AccountEditor() } }
        .sheet(isPresented: $quickAdd) { NavigationStack { QuickAddView() } }
        .sheet(isPresented: $spendingSetup, onDismiss: { Task { await loadOverview() } }) { NavigationStack { SpendingSettingsView() } }
    }
    private func overviewValue(_ name: String) -> JSONValue {
        switch name {
        case "Net worth": return netWorth["netWorth"]
        case "Spending": return spending["current"]["outflow"]
        default: return section == "Investments" ? displayed : model.valuation["summary"]["totalValueBase"]
        }
    }
    private var valueHeader: some View {
        let names = ["Investments", "Net worth", "Spending"]
        let cards = DashboardMetricCards(
            values: names.map { name in
                name == "Spending" && !spendingEnabled ? "Set up" : money(overviewValue(name), currency: model.currency, hidden: model.hideBalances)
            },
            active: names.firstIndex(of: section) ?? 0,
            dark: scheme == .dark, fontSize: balanceSize,
            stacked: textSize.isAccessibilitySize, reduceMotion: reduceMotion
        ) { index in
            let name = names[index]
            guard name != section else { return }
            if name == "Spending" && !spendingEnabled { spendingSetup = true }
            else { selectedDate = nil; section = name }
        }
        return VStack(alignment: .leading, spacing: 8) {
            cards.frame(height: cards.height)
            if let overviewError { Text(overviewError).font(.caption).foregroundStyle(.secondary) }
        }.frame(maxWidth: .infinity, alignment: .leading)
    }
    private func loadOverview() async {
        overviewError = nil
        do { netWorth = try await model.engine.request("/api/v1/net-worth") }
        catch { overviewError = error.localizedDescription }
        do {
            let settings = try await model.engine.request("/api/v1/spending/settings")
            spendingEnabled = settings["enabled"].flag
            if !spendingEnabled && section == "Spending" { section = "Investments" }
            if spendingEnabled {
                var calendar = Calendar(identifier: .gregorian)
                calendar.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current
                if let month = calendar.dateInterval(of: .month, for: .now) {
                    spending = try await model.engine.request("/api/v1/spending/report", method: "POST", body: .strings([
                        "startDate": ISO8601DateFormatter().string(from: month.start),
                        "endDate": ISO8601DateFormatter().string(from: month.end.addingTimeInterval(-1))]))
                }
            }
        } catch { overviewError = error.localizedDescription }
    }
    private var investmentsPage: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                valueHeader
                if let chosen = selection { Text(chosen.date.formatted(date: .abbreviated, time: .omitted)).font(.caption).foregroundStyle(.secondary) }
                if model.hideBalances {
                    ContentUnavailableView("Balances hidden", systemImage: "eye.slash").frame(height: 220)
                } else if points.isEmpty {
                    ContentUnavailableView("No history yet", systemImage: "chart.xyaxis.line", description: Text("Your portfolio history will appear here as valuations become available."))
                } else { historyChart }
                periodSelector
                let layout = textSize.isAccessibilitySize ? AnyLayout(VStackLayout(alignment: .leading, spacing: 20)) : AnyLayout(HStackLayout(alignment: .top, spacing: 24))
                layout {
                    metric("Cash balance", key: "cashBalanceBase")
                    metric("Investments", key: "investmentMarketValueBase")
                }.padding(20).frame(maxWidth: .infinity, alignment: .leading)
                    .background(scheme == .dark ? Color(red: 0.11, green: 0.165, blue: 0.188) : .white, in: .rect(cornerRadius: 22))
                accountsSection
                GlassEffectContainer(spacing: 12) {
                    HStack(spacing: 12) {
                        NavigationLink { HoldingsView() } label: { Label("Holdings", systemImage: "briefcase").font(.subheadline).frame(minHeight: 32) }.buttonStyle(.glass)
                        NavigationLink { NetWorthView() } label: { Label("Net worth", systemImage: "wallet.bifold").font(.subheadline).frame(minHeight: 32) }.buttonStyle(.glass)
                    }.controlSize(.regular)
                }
                Text("Balance change includes deposits and withdrawals. See Insights for investment performance.")
                    .font(.footnote).foregroundStyle(.secondary)
            }.padding(.horizontal, 24).padding(.top, 20).padding(.bottom, 24)
                .frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(canvas)
        .refreshable { await model.refresh(); await model.synchronize() }
    }
    private var historyChart: some View {
        let history = points
        let bounds = range
        let chosen = selection
        let markedDate = chosen?.date ?? history.last?.date
        let plot = Chart(history) { point in
            historyMarks(point, baseline: bounds.lowerBound)
            if point.date == markedDate {
                PointMark(x: .value("Date", point.date), y: .value("Value", point.plotted))
                    .foregroundStyle(accent).symbolSize(24)
                if chosen != nil {
                    RuleMark(x: .value("Date", point.date))
                        .foregroundStyle(Color.secondary.opacity(0.45)).lineStyle(StrokeStyle(lineWidth: 1, dash: [3,4]))
                }
            }
        }
        return plot
            .chartYScale(domain: bounds)
            .chartYAxis { AxisMarks(position: .trailing, values: .automatic(desiredCount: 3)) { value in
                AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5, dash: [2,6])).foregroundStyle(Color.secondary.opacity(0.13))
                AxisValueLabel { if let number = value.as(Double.self) { Text(axisLabel(number)).font(.caption2).foregroundStyle(.secondary) } }
            } }
            .chartXAxis { AxisMarks(values: .automatic(desiredCount: 3)) { _ in AxisValueLabel(format: .dateTime.day().month(.abbreviated)) } }
            .chartXSelection(value: $selectedDate)
            .frame(height: 210)
            .accessibilityLabel("Portfolio value history in " + model.currency)
            .accessibilityAdjustableAction { adjustSelection($0) }
    }
    private func axisLabel(_ value: Double) -> String {
        if model.currency == "INR", abs(value) >= 100_000 {
            return (value / 100_000).formatted(.number.precision(.fractionLength(0...1))) + "L"
        }
        return value.formatted(.number.notation(.compactName))
    }
    private func adjustSelection(_ direction: AccessibilityAdjustmentDirection) {
        let history: [DashboardPoint] = points
        guard !history.isEmpty else { return }
        var index: Int = history.count - 1
        if let chosen = selection, let match = history.firstIndex(where: { $0.date == chosen.date }) { index = match }
        switch direction {
        case .increment: index = min(history.count - 1, index + 1)
        case .decrement: index = max(0, index - 1)
        @unknown default: return
        }
        selectedDate = history[index].date
    }
    @ChartContentBuilder
    private func historyMarks(_ point: DashboardPoint, baseline: Double) -> some ChartContent {
        AreaMark(x: .value("Date", point.date), yStart: .value("Baseline", baseline), yEnd: .value("Value", point.plotted))
            .interpolationMethod(.monotone)
            .foregroundStyle(LinearGradient(colors: [accent.opacity(0.18), accent.opacity(0.005)], startPoint: .top, endPoint: .bottom))
        LineMark(x: .value("Date", point.date), y: .value("Value", point.plotted))
            .foregroundStyle(accent)
            .interpolationMethod(.monotone)
            .lineStyle(StrokeStyle(lineWidth: 2, lineCap: .round, lineJoin: .round))
    }
    private var periodSelector: some View {
        GlassEffectContainer(spacing: 8) {
            ViewThatFits(in: .horizontal) {
                HStack(spacing: 0) { periodButtons }.frame(maxWidth: .infinity)
                ScrollView(.horizontal) { HStack(spacing: 8) { periodButtons }.fixedSize(horizontal: true, vertical: false) }.scrollIndicators(.hidden)
            }
        }
    }
    private var periodButtons: some View {
        ForEach([7,30,90,180,365,0], id: \.self) { value in
            Button {
                withAnimation(reduceMotion ? nil : .smooth(duration: 0.22)) { period = value; selectedDate = nil }
            } label: {
                Text([7:"1W",30:"1M",90:"3M",180:"6M",365:"1Y",0:"All"][value]!)
                    .font(.subheadline.weight(period == value ? .semibold : .regular))
                    .frame(width: circleSize, height: circleSize)
                    .contentShape(.circle)
                    .glassEffect(period == value ? .regular.interactive() : .identity, in: .circle)
                    .glassEffectID(value, in: periodGlass)
            }.buttonStyle(.plain).frame(minWidth: circleSize, maxWidth: .infinity)
                .accessibilityAddTraits(period == value ? .isSelected : [])
        }
    }
    private func metric(_ title: String, key: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title).font(.subheadline).foregroundStyle(.secondary)
            Text(money(model.valuation["summary"][key], currency: model.currency, hidden: model.hideBalances)).font(.title3.weight(.semibold)).monospacedDigit()
        }.frame(maxWidth: .infinity, alignment: .leading)
    }
    private var accountsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Accounts").font(.title2.weight(.semibold))
                Text("\(model.accounts.count)").font(.subheadline).foregroundStyle(.secondary)
                Spacer()
                Button { adding = true } label: {
                    Image(systemName: "plus").font(.subheadline)
                        .frame(width: circleSize, height: circleSize)
                        .contentShape(.circle)
                        .glassEffect(.regular.interactive(), in: .circle)
                }.buttonStyle(.plain).accessibilityLabel("Add account")
            }
            if model.accounts.isEmpty {
                ContentUnavailableView("Your portfolio starts here", systemImage: "wallet.bifold", description: Text("Connect to your server or add an account to work offline."))
                NavigationLink("Connect to your server") { ServerSyncView() }.buttonStyle(.glass)
            }
            ForEach(model.accounts) { account in
                NavigationLink { AccountDetailView(account: account) } label: {
                    HStack(spacing: 12) {
                        Image(systemName: account["accountType"].text == "CASH" ? "building.columns" : "chart.xyaxis.line")
                            .font(.body).foregroundStyle(accent).frame(width: 40,height: 40)
                            .background(accent.opacity(0.1), in: .rect(cornerRadius: 12)).accessibilityHidden(true)
                        VStack(alignment: .leading, spacing: 5) { Text(account["name"].text).font(.body.weight(.medium)); Text(account["currency"].text).font(.caption).foregroundStyle(.secondary) }
                        Spacer(minLength: 8)
                        if let value = model.valuation["accounts"].values.first(where: { $0["accountId"].text == account.id }) {
                            Text(money(value["totalValue"], currency: account["currency"].text, hidden: model.hideBalances)).font(.subheadline.weight(.semibold)).monospacedDigit()
                        }
                        Image(systemName: "chevron.right").font(.caption2.weight(.semibold)).foregroundStyle(.tertiary).accessibilityHidden(true)
                    }.padding(.vertical, 12).contentShape(.rect)
                }.buttonStyle(.plain)
            }
        }
    }
}

struct AccountDetailView: View {
    let account: Record
    @State private var editing = false
    var body: some View {
        List {
            Section {
                LabeledContent("Currency", value: account["currency"].text)
                LabeledContent("Type", value: account["accountType"].text.capitalized)
                LabeledContent("Tracking", value: account["trackingMode"].text.capitalized)
            }
            NavigationLink("Transactions") { ActivitiesView(accountID: account.id) }
            NavigationLink("Holdings") { HoldingsView(accountID: account.id) }
        }.navigationTitle(account["name"].text)
        .toolbar { Button("Edit") { editing = true } }
        .sheet(isPresented: $editing) { NavigationStack { AccountEditor(account: account) } }
    }
}

struct AccountEditor: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    var account: Record?
    @State private var name = ""
    @State private var currency = "USD"
    @State private var kind = "SECURITIES"
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        Form {
            TextField("Account name", text: $name)
            TextField("Currency", text: $currency).disabled(account != nil).textInputAutocapitalization(.characters).autocorrectionDisabled()
            Picker("Type", selection: $kind) {
                ForEach(["SECURITIES", "CASH", "CRYPTOCURRENCY", "OTHER"], id: \.self) { Text($0.capitalized).tag($0) }
            }
            if let error { Text(error).foregroundStyle(.red) }
        }.navigationTitle(account == nil ? "New account" : "Edit account")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) { Button("Save") { Task { await save() } }.disabled(saving || name.trimmingCharacters(in: .whitespaces).isEmpty) }
        }
        .onAppear {
            name = account?["name"].text ?? ""
            currency = account?["currency"].text ?? model.currency
            kind = account?["accountType"].text ?? "SECURITIES"
        }
    }
    private func save() async {
        saving = true; defer { saving = false }
        var fields = account?.value.fields ?? [:]
        fields["name"] = .string(name); fields["currency"] = .string(currency.uppercased()); fields["accountType"] = .string(kind)
        if account == nil {
            fields["isDefault"] = .bool(model.accounts.isEmpty); fields["isActive"] = .bool(true)
            fields["trackingMode"] = .string("TRANSACTIONS")
        }
        do {
            _ = try await model.mutate("/api/v1/accounts" + (account.map { "/" + $0.id } ?? ""),
                method: account == nil ? "POST" : "PUT", body: .object(fields)); dismiss()
        } catch { self.error = error.localizedDescription }
    }
}
