import SwiftUI

struct SetupView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var currency = Locale.current.currency?.identifier ?? "USD"
    @State private var timezone = TimeZone.current.identifier
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                VStack(alignment: .leading, spacing: 16) {
                    Image(systemName: "chart.xyaxis.line").font(.title).foregroundStyle(palette.accent)
                        .frame(width: 64, height: 64).background(palette.accent.opacity(0.1), in: .rect(cornerRadius: 20)).accessibilityHidden(true)
                    Text("Your portfolio. On your device.").font(.system(.largeTitle, design: .rounded).weight(.semibold))
                    Text("Choose how you want to view your portfolio. You can connect to your existing server after setup, or use Steadyfolio offline.")
                        .font(.body).foregroundStyle(.secondary)
                }
                VStack(alignment: .leading, spacing: 4) {
                    Text("Display preferences").font(.title3.weight(.semibold)).padding(.bottom, 8)
                    LabeledContent("Base currency") {
                        Picker("Base currency", selection: $currency) { ForEach(Locale.commonISOCurrencyCodes, id: \.self) { Text($0).tag($0) } }.labelsHidden()
                    }
                    Divider()
                    LabeledContent("Time zone") {
                        Picker("Time zone", selection: $timezone) { ForEach(timeZoneChoices(including: timezone), id: \.self) { Text($0).tag($0) } }.labelsHidden()
                    }
                }.padding(20).background(palette.surface, in: .rect(cornerRadius: 22))
                if let error { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }
            }.padding(.horizontal, 24).padding(.top, 48).padding(.bottom, 24).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.background(palette.canvas).tint(palette.accent)
        .safeAreaInset(edge: .bottom) {
            Button { Task { await save() } } label: {
                Text(saving ? "Saving…" : "Continue").font(.body.weight(.semibold)).frame(maxWidth: .infinity, minHeight: 36)
            }.buttonStyle(.glassProminent).tint(palette.accent).disabled(saving)
                .padding(.horizontal, 24).padding(.vertical, 12).frame(maxWidth: 720)
        }
    }
    private func save() async {
        saving = true; defer { saving = false }
        do {
            _ = try await model.mutate("/api/v1/settings", method: "PUT", body: .object([
                "baseCurrency": .string(currency), "timezone": .string(timezone), "onboardingCompleted": .bool(true)]))
        } catch { self.error = error.localizedDescription }
    }
}
