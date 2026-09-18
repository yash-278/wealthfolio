import SwiftUI

struct MoreView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    private var syncSummary: String {
        guard model.connected else { return "Stored on this device" }
        if model.sync["connection"]["paused"].flag { return "Sync paused" }
        let pending = model.sync["connection"]["pending"].text
        return pending.isEmpty || pending == "0" ? "Connected" : "Connected · \(pending) pending"
    }
    var body: some View {
        List {
            Section {
                row("Quick Add", symbol: "plus.circle", tint: palette.accent) { QuickAddView() }
            }.surfaceRows()
            Section("Portfolio") {
                row("Holdings", symbol: "briefcase", tint: palette.accent) { HoldingsView() }
                row("Net worth", symbol: "wallet.bifold", tint: palette.blue) { NetWorthView() }
                row("Spending", symbol: "creditcard", tint: palette.accent) { SpendingView() }
                row("Goals", symbol: "target", tint: palette.blue) { GoalsView() }
            }.surfaceRows()
            Section("App") {
                row("Your server", detail: syncSummary, symbol: "arrow.triangle.2.circlepath", tint: palette.accent) { ServerSyncView() }
                row("Settings", symbol: "gearshape", tint: palette.blue) { SettingsView() }
            }.surfaceRows()
        }.listStyle(.insetGrouped).themedForm().leadingPageTitle("More")
    }
    private func row<Destination: View>(_ title: String, detail: String = "", symbol: String, tint: Color, @ViewBuilder destination: @escaping () -> Destination) -> some View {
        NavigationLink(destination: destination) {
            HStack(spacing: 12) {
                IconTile(symbol: symbol, tint: tint)
                VStack(alignment: .leading, spacing: 4) {
                    Text(title).font(.body.weight(.medium))
                    if !detail.isEmpty { Text(detail).font(.caption).foregroundStyle(.secondary) }
                }
            }.padding(.vertical, 2)
        }
    }
}

struct SettingsView: View {
    @Environment(AppModel.self) private var model
    @State private var currency = ""
    @State private var timezone = TimeZone.current.identifier
    @State private var saving = false
    var body: some View {
        Form {
            Section {
                NavigationLink { ServerSyncView() } label: { Label("Your server", systemImage: "arrow.triangle.2.circlepath") }
            } footer: { Text("Your portfolio is stored on this device. Server synchronization is optional.") }.surfaceRows()
            Section("Portfolio") {
                LabeledContent("Base currency") { TextField("USD", text: $currency).multilineTextAlignment(.trailing).textInputAutocapitalization(.characters).autocorrectionDisabled() }
                Picker("Time zone", selection: $timezone) { ForEach(timeZoneChoices(including: timezone), id: \.self) { Text($0).tag($0) } }
                Toggle("Hide balances", isOn: Binding(get: { model.hideBalances }, set: { model.hideBalances = $0 }))
                Button("Save preferences") {
                    Task {
                        saving = true; defer { saving = false }
                        do { _ = try await model.mutate("/api/v1/settings", method: "PUT", body: .strings(["baseCurrency": currency.uppercased(), "timezone": timezone])) }
                        catch { model.error = error.localizedDescription }
                    }
                }.disabled(saving || currency.isEmpty)
            }.surfaceRows()
            Section("Data") { NavigationLink { ExportView() } label: { Label("Export CSV", systemImage: "square.and.arrow.up") } }.surfaceRows()
            Section("Assistant") { NavigationLink { AIProvidersView() } label: { Label("AI providers", systemImage: "sparkles") } }.surfaceRows()
            Section {
                LabeledContent("Version", value: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "")
                Link(destination: URL(string: "https://github.com/yash-278/wealthfolio")!) { Label("Source code (AGPL-3.0)", systemImage: "chevron.left.forwardslash.chevron.right") }
            } header: { Text("About") } footer: {
                Text("Steadyfolio is an independent fork based on Wealthfolio (https://wealthfolio.app) and is not affiliated with or endorsed by the official project. Wealthfolio is a trademark of Teymz Inc.")
            }.surfaceRows()
        }.themedForm().leadingPageTitle("Settings")
        .onAppear { currency = model.currency; timezone = model.settings["timezone"].text }
    }
}

struct ServerSyncView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var endpoint = ""
    @State private var password = ""
    @State private var working = false
    @State private var error: String?
    @State private var accepting = false
    private var paused: Bool { model.sync["connection"]["paused"].flag }
    private var lastSync: String {
        let text = model.sync["connection"]["lastSync"].text
        if text.isEmpty { return "Never" }
        return parseActivityDate(text)?.formatted(.relative(presentation: .named)) ?? text
    }
    var body: some View {
        Form {
            Section {
                HStack(spacing: 12) {
                    IconTile(symbol: model.connected ? (paused ? "pause.circle" : "checkmark.icloud") : "iphone", tint: palette.accent)
                    VStack(alignment: .leading, spacing: 4) {
                        Text(model.connected ? (paused ? "Sync paused" : "Connected") : "Stored on this device").font(.body.weight(.medium))
                        Text(model.connected ? model.sync["connection"]["endpoint"].text : "Server synchronization is optional.")
                            .font(.caption).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                    }
                    Spacer()
                    if working || model.syncing { ProgressView().controlSize(.small) }
                }.padding(.vertical, 2)
            }.surfaceRows()
            Section {
                TextField("https://your-server", text: $endpoint).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                    .disabled(model.connected)
                SecureField("Website password", text: $password)
                Button(model.connected ? "Sign in again" : "Connect and download") { Task { await connect() } }
                    .disabled(working || endpoint.isEmpty || password.isEmpty)
            } header: { Text("Connection") } footer: {
                if !model.connected { Text("Connect before creating local accounts to download your existing portfolio. Existing local records are never overwritten to pair a server.") }
            }.surfaceRows()
            if model.connected {
                Section("Synchronization") {
                    LabeledContent("Pending changes", value: model.sync["connection"]["pending"].text)
                    LabeledContent("Last sync", value: lastSync)
                    Button("Sync now", systemImage: "arrow.triangle.2.circlepath") { Task { await model.synchronize() } }.disabled(model.syncing)
                    Button(paused ? "Resume sync" : "Pause sync", systemImage: paused ? "play" : "pause") {
                        Task {
                            do { model.sync = try await model.engine.request("/native/sync/pause", method: "POST", body: .object(["paused": .bool(!paused)])) }
                            catch { self.error = error.localizedDescription }
                        }
                    }
                }.surfaceRows()
            }
            if model.sync["conflict"] != .null {
                Section {
                    Label(model.sync["conflict"]["label"].text, systemImage: "exclamationmark.triangle").foregroundStyle(.orange)
                    Button("Use server version", role: .destructive) { accepting = true }
                } header: { Text("Change needs review") } footer: {
                    Text("The same record changed on this device and on the server. Accepting the server version discards the conflicting local edit.")
                }.surfaceRows()
            }
            if let error { Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }.surfaceRows() }
            if let syncError = model.syncError { Section { Text(syncError).font(.subheadline).foregroundStyle(.secondary) }.surfaceRows() }
            if !model.sync["error"].text.isEmpty { Section { Text(model.sync["error"].text).font(.subheadline).foregroundStyle(.secondary) }.surfaceRows() }
        }.themedForm().leadingPageTitle("Your server")
        .task {
            do { model.sync = try await model.engine.request("/native/sync/status"); endpoint = model.sync["connection"]["endpoint"].text }
            catch { self.error = error.localizedDescription }
        }
        .confirmationDialog("Replace the conflicting local edit?", isPresented: $accepting) {
            Button("Use server version", role: .destructive) {
                Task {
                    do {
                        model.sync = try await model.engine.request("/native/sync/accept-server", method: "POST", body: .strings([
                            "eventId": model.sync["conflict"]["eventId"].text, "serverEventId": model.sync["conflict"]["serverEventId"].text]))
                        await model.refresh()
                    } catch { self.error = error.localizedDescription }
                }
            }
        }
    }
    private func connect() async {
        working = true; defer { working = false; password = "" }
        do { model.sync = try await model.engine.request("/native/sync/connect", method: "POST", body: .strings(["endpoint": endpoint, "password": password])); error = nil; await model.refresh() }
        catch { self.error = error.localizedDescription }
    }
}

struct AIProvidersView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var providers: [Record] = []
    @State private var error: String?
    var body: some View {
        List {
            if let error { Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }.surfaceRows() }
            Section {
                ForEach(providers) { provider in
                    NavigationLink { AIProviderEditor(provider: provider) } label: {
                        HStack(spacing: 12) {
                            IconTile(symbol: provider["enabled"].flag ? "sparkles" : "circle.dashed", tint: provider["enabled"].flag ? palette.accent : .secondary)
                            VStack(alignment: .leading, spacing: 4) {
                                Text(provider["name"].text.isEmpty ? provider.id : provider["name"].text).font(.body.weight(.medium))
                                Text(status(provider)).font(.caption).foregroundStyle(.secondary)
                            }
                        }.padding(.vertical, 2)
                    }
                }
            } footer: { Text("API keys are stored in the Keychain on this device.") }.surfaceRows()
        }.listStyle(.insetGrouped).themedForm().leadingPageTitle("AI providers").task {
            do { let data = try await model.engine.request("/api/v1/ai/providers"); providers = data["providers"].values.map(Record.init); error = nil }
            catch { self.error = error.localizedDescription }
        }
    }
    private func status(_ provider: Record) -> String {
        if provider["isDefault"].flag { return "Default" }
        if provider["enabled"].flag { return "Enabled" }
        return provider["hasApiKey"].flag ? "Key saved · off" : "Not set up"
    }
}

struct AIProviderEditor: View {
    let provider: Record
    @Environment(AppModel.self) private var model
    @State private var key = ""
    @State private var selectedModel = ""
    @State private var customURL = ""
    @State private var enabled = false
    @State private var message: String?
    var body: some View {
        Form {
            Section { Toggle("Enabled", isOn: $enabled) }.surfaceRows()
            Section("Connection") {
                LabeledContent("Model") { TextField(provider["defaultModel"].text, text: $selectedModel).multilineTextAlignment(.trailing).textInputAutocapitalization(.never).autocorrectionDisabled() }
                LabeledContent("Endpoint") { TextField("Optional", text: $customURL).multilineTextAlignment(.trailing).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled() }
                SecureField(provider["hasApiKey"].flag ? "API key saved · enter to replace" : "API key", text: $key)
            }.surfaceRows()
            Section {
                Button("Save provider") {
                    Task {
                        do {
                            if !key.isEmpty {
                                _ = try await model.engine.request("/api/v1/secrets", method: "POST", body: .strings(["secretKey": "ai_" + provider.id, "secret": key]))
                                key = ""
                            }
                            _ = try await model.engine.request("/api/v1/ai/providers/settings", method: "PUT", body: .object([
                                "providerId": .string(provider.id), "enabled": .bool(enabled), "selectedModel": .string(selectedModel),
                                "customUrl": customURL.isEmpty ? .null : .string(customURL)]))
                            message = "Provider settings saved."
                        } catch { message = error.localizedDescription }
                    }
                }
                Button("Use as default") { Task {
                    do { _ = try await model.engine.request("/api/v1/ai/providers/default", method: "POST", body: .strings(["providerId": provider.id])); message = "Default provider updated." }
                    catch { message = error.localizedDescription }
                } }
            } footer: { if let message { Text(message) } }.surfaceRows()
        }.themedForm().leadingPageTitle(provider["name"].text).onAppear {
            selectedModel = provider["selectedModel"].text.isEmpty ? provider["defaultModel"].text : provider["selectedModel"].text
            customURL = provider["customUrl"].text; enabled = provider["enabled"].flag
        }
    }
}
