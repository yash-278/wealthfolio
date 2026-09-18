import SwiftUI

struct SetupView: View {
    @Environment(AppModel.self) private var model
    @State private var currency = Locale.current.currency?.identifier ?? "USD"
    @State private var timezone = TimeZone.current.identifier
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Image(systemName: "chart.xyaxis.line").font(.system(size: 44)).foregroundStyle(.tint).padding(.vertical)
                    Text("Your portfolio. On your device.").font(.title2.bold())
                    Text("Choose how you want to view your portfolio. You can connect to your existing server after setup, or use Wealthfolio offline.")
                }
                Section("Display preferences") {
                    Picker("Base currency", selection: $currency) { ForEach(Locale.commonISOCurrencyCodes, id: \.self) { Text($0).tag($0) } }
                    Picker("Time zone", selection: $timezone) { ForEach(TimeZone.knownTimeZoneIdentifiers, id: \.self) { Text($0).tag($0) } }
                }
                if let error { Text(error).foregroundStyle(.red) }
                Button("Continue") { Task {
                    saving = true; defer { saving = false }
                    do {
                        _ = try await model.mutate("/api/v1/settings", method: "PUT", body: .object([
                            "baseCurrency": .string(currency), "timezone": .string(timezone), "onboardingCompleted": .bool(true)]))
                    } catch { self.error = error.localizedDescription }
                } }.disabled(saving)
            }.navigationTitle("Welcome")
        }
    }
}
