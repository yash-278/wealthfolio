import SwiftUI

struct QuickAddView: View {
    @Environment(AppModel.self) private var model
    @State private var text = ""
    @State private var requestID = UUID().uuidString
    @State private var saving = false
    @State private var receipt: JSONValue = .null
    @State private var reviews: [Record] = []
    @State private var selected: Record?
    @State private var error: String?
    var body: some View {
        List {
            Section("Paste an alert or describe a transaction") {
                TextEditor(text: $text).frame(minHeight: 120)
                Button("Save for processing", systemImage: "plus.circle.fill") { Task { await capture() } }.disabled(saving || text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                if saving { ProgressView() }
            }
            if receipt != .null {
                Section("Latest capture") {
                    Text(receipt["status"].text.replacingOccurrences(of: "_", with: " ").capitalized)
                    Button("Review capture") { selected = Record(receipt) }
                }
            }
            if let error { Text(error).foregroundStyle(.red) }
            Section("Needs review") {
                ForEach(reviews) { capture in
                    Button { selected = capture } label: {
                        VStack(alignment: .leading, spacing: 5) {
                            Text(capture["input"]["text"].text).lineLimit(3).foregroundStyle(.primary)
                            Text(capture["status"].text.replacingOccurrences(of: "_", with: " ").capitalized).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
                if reviews.isEmpty { Text("No captures need review.").foregroundStyle(.secondary) }
            }
        }.navigationTitle("Quick Add")
        .toolbar { NavigationLink { QuickAddSettingsView() } label: { Label("Settings", systemImage: "gearshape") } }
        .task(id: model.revision) { await load() }
        .refreshable { await load() }
        .sheet(item: $selected) { capture in NavigationStack { CaptureReviewView(capture: capture) } }
    }
    private func capture() async {
        saving = true; defer { saving = false }
        do {
            receipt = try await model.mutate("/api/v1/captures", body: .strings(["clientRequestId": requestID, "text": text, "inputKind": "typed_note"]))
            text = ""; requestID = UUID().uuidString; error = nil; await load()
        } catch { self.error = error.localizedDescription }
    }
    private func load() async {
        do {
            reviews = try await model.engine.request("/api/v1/capture-reviews").values.map(Record.init)
            if !receipt["id"].text.isEmpty { receipt = try await model.engine.request("/api/v1/captures/" + receipt["id"].text) }
        } catch { self.error = error.localizedDescription }
    }
}
struct CaptureReviewView: View {
    let capture: Record
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    @State private var current: JSONValue = .null
    @State private var fields: [String: JSONValue] = [:]
    @State private var error: String?
    @State private var saving = false
    private func binding(_ key: String) -> Binding<String> {
        Binding(get: { fields[key]?.text ?? "" }, set: { fields[key] = .string($0) })
    }
    var body: some View {
        Form {
            Section { Text(current["input"]["text"].text); Text(current["status"].text).foregroundStyle(.secondary) }
            Section("Transaction") {
                Picker("Account", selection: binding("accountId")) { ForEach(model.accounts) { Text($0["name"].text).tag($0.id) } }
                TextField("Amount", text: binding("amount")).keyboardType(.decimalPad)
                TextField("Currency", text: binding("currency"))
                TextField("Date (YYYY-MM-DD)", text: binding("date"))
                Picker("Direction", selection: binding("direction")) { Text("Debit").tag("debit"); Text("Credit").tag("credit") }
                TextField("Merchant", text: binding("merchant"))
                TextField("Reference", text: binding("reference"))
            }
            if let error { Text(error).foregroundStyle(.red) }
            Section {
                Button("Confirm transaction") { Task { await resolve() } }.disabled(saving || fields.isEmpty)
                Button("Retry extraction") { Task { await perform("/captures/" + capture.id + "/retry") } }.disabled(saving)
                Button("Dismiss capture", role: .destructive) { Task { await perform("/capture-reviews/" + capture.id + "/dismiss") } }.disabled(saving)
            }
        }.navigationTitle("Review capture").toolbar { Button("Close") { dismiss() } }
        .task {
            do {
                current = try await model.engine.request("/api/v1/captures/" + capture.id)
                fields = current["candidates"].values.first?["fields"].fields ?? [:]
            } catch { self.error = error.localizedDescription }
        }
    }
    private func resolve() async {
        saving = true; defer { saving = false }
        do {
            _ = try await model.mutate("/api/v1/capture-reviews/" + capture.id + "/resolve", body: .object(["version": current["version"], "fields": .object(fields), "action": .string("complete")]))
            dismiss()
        } catch { self.error = error.localizedDescription }
    }
    private func perform(_ path: String) async {
        saving = true; defer { saving = false }
        do { _ = try await model.mutate("/api/v1" + path, body: .object(["version": current["version"]])); dismiss() }
        catch { self.error = error.localizedDescription }
    }
}
struct QuickAddSettingsView: View {
    @Environment(AppModel.self) private var model
    @State private var settings: [String: JSONValue] = [:]
    @State private var error: String?
    private func binding(_ key: String) -> Binding<String> { Binding(get: { settings[key]?.text ?? "" }, set: { settings[key] = .string($0) }) }
    var body: some View {
        Form {
            TextField("Provider", text: binding("provider")).textInputAutocapitalization(.never)
            TextField("Model", text: binding("model")).textInputAutocapitalization(.never)
            Picker("Default account", selection: binding("typedNoteAccountId")) {
                Text("Choose an account").tag("")
                ForEach(model.accounts) { Text($0["name"].text).tag($0.id) }
            }
            TextField("Time zone", text: binding("timezone"))
            NavigationLink("Configure provider credentials") { AIProvidersView() }
            if let error { Text(error).foregroundStyle(.red) }
            Button("Save settings") { Task {
                do { _ = try await model.mutate("/api/v1/quick-add/settings", method: "PUT", body: .object(settings)) }
                catch { self.error = error.localizedDescription }
            } }.disabled(settings.isEmpty)
        }.navigationTitle("Quick Add settings").task {
            do { settings = try await model.engine.request("/api/v1/quick-add/settings").fields } catch { self.error = error.localizedDescription }
        }
    }
}
