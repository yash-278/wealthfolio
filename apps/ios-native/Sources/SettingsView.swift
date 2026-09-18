import SwiftUI

struct MoreView: View {
    var body: some View {
        List {
            NavigationLink { QuickAddView() } label: { Label("Quick Add", systemImage: "plus.circle") }
            NavigationLink { HoldingsView() } label: { Label("Holdings", systemImage: "briefcase") }
            NavigationLink { NetWorthView() } label: { Label("Net worth", systemImage: "wallet.bifold") }
            NavigationLink { SpendingView() } label: { Label("Spending", systemImage: "banknote") }
            NavigationLink { GoalsView() } label: { Label("Goals", systemImage: "target") }
            NavigationLink { SettingsView() } label: { Label("Settings", systemImage: "gearshape") }
        }.navigationTitle("More")
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
                Text("Your portfolio is stored on this device. Server synchronization is optional.").font(.footnote).foregroundStyle(.secondary)
            }
            Section("Portfolio") {
                TextField("Base currency", text: $currency).textInputAutocapitalization(.characters)
                Picker("Time zone", selection: $timezone) { ForEach(TimeZone.knownTimeZoneIdentifiers, id: \.self) { Text($0).tag($0) } }
                Button("Save preferences") {
                    Task {
                        saving = true; defer { saving = false }
                        do { _ = try await model.mutate("/api/v1/settings", method: "PUT", body: .strings(["baseCurrency": currency.uppercased(), "timezone": timezone])) }
                        catch { model.error = error.localizedDescription }
                    }
                }.disabled(saving)
            }
            Section("Data") { NavigationLink("Export CSV") { ExportView() } }
            Section("Assistant") { NavigationLink("AI providers") { AIProvidersView() } }
        }.navigationTitle("Settings")
        .onAppear { currency = model.currency; timezone = model.settings["timezone"].text }
    }
}

struct ServerSyncView: View {
    @Environment(AppModel.self) private var model
    @State private var endpoint = ""
    @State private var password = ""
    @State private var working = false
    @State private var error: String?
    @State private var accepting = false
    var body: some View {
        Form {
            Section("Connection") {
                TextField("https://your-server", text: $endpoint).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                    .disabled(model.connected)
                SecureField("Website password", text: $password)
                Button(model.connected ? "Sign in again" : "Connect and download") { Task { await connect() } }
                    .disabled(working || endpoint.isEmpty || password.isEmpty)
            }
            if model.connected {
                Section("Synchronization") {
                    LabeledContent("Pending changes", value: model.sync["connection"]["pending"].text)
                    LabeledContent("Last sync", value: model.sync["connection"]["lastSync"].text)
                    Button("Sync now", systemImage: "arrow.triangle.2.circlepath") { Task { await model.synchronize() } }.disabled(model.syncing)
                    Button(model.sync["connection"]["paused"].flag ? "Resume sync" : "Pause sync") {
                        Task {
                            do { model.sync = try await model.engine.request("/native/sync/pause", method: "POST", body: .object(["paused": .bool(!model.sync["connection"]["paused"].flag)])) }
                            catch { self.error = error.localizedDescription }
                        }
                    }
                }
            } else {
                Section { Text("Connect before creating local accounts to download your existing portfolio. Existing local records are never overwritten to pair a server.") }
            }
            if model.sync["conflict"] != .null {
                Section("Change needs review") {
                    Text(model.sync["conflict"]["label"].text)
                    Text("The same record changed on this device and on the server. Accepting the server version discards the conflicting local edit.").font(.footnote)
                    Button("Use server version", role: .destructive) { accepting = true }
                }
            }
            if let error { Section { Text(error).foregroundStyle(.red) } }
            if let syncError = model.syncError { Section { Text(syncError).foregroundStyle(.secondary) } }
            if !model.sync["error"].text.isEmpty { Section { Text(model.sync["error"].text).foregroundStyle(.secondary) } }
            if working { ProgressView("Connecting…") }
        }.navigationTitle("Your server")
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
    @State private var providers: [Record] = []
    @State private var error: String?
    var body: some View {
        List {
            if let error { Text(error).foregroundStyle(.red) }
            ForEach(providers) { provider in
                NavigationLink(provider["name"].text.isEmpty ? provider.id : provider["name"].text) { AIProviderEditor(provider: provider) }
            }
        }.navigationTitle("AI providers").task {
            do { let data = try await model.engine.request("/api/v1/ai/providers"); providers = data["providers"].values.map(Record.init) }
            catch { self.error = error.localizedDescription }
        }
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
            Toggle("Enabled", isOn: $enabled)
            TextField("Model", text: $selectedModel).textInputAutocapitalization(.never).autocorrectionDisabled()
            TextField("Custom endpoint (optional)", text: $customURL).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
            SecureField("API key", text: $key)
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
            if let message { Text(message) }
        }.navigationTitle(provider["name"].text).onAppear {
            selectedModel = provider["selectedModel"].text.isEmpty ? provider["defaultModel"].text : provider["selectedModel"].text
            customURL = provider["customUrl"].text; enabled = provider["enabled"].flag
        }
    }
}
