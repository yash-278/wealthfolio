import SwiftUI

struct QuickAddView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var text = ""
    @State private var requestID = UUID().uuidString
    @State private var saving = false
    @State private var receipt: JSONValue = .null
    @State private var reviews: [Record] = []
    @State private var selected: Record?
    @State private var error: String?
    private func status(_ value: JSONValue) -> String { value["status"].text.replacingOccurrences(of: "_", with: " ").capitalized }
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Paste an alert or describe a transaction").font(.subheadline).foregroundStyle(.secondary)
                    TextEditor(text: $text).frame(minHeight: 120).scrollContentBackground(.hidden)
                        .padding(12).background(palette.surface, in: .rect(cornerRadius: 22))
                        .accessibilityLabel("Transaction text")
                    Button { Task { await capture() } } label: {
                        HStack(spacing: 8) {
                            if saving { ProgressView().controlSize(.small) }
                            Label(saving ? "Saving…" : "Save for processing", systemImage: "plus.circle.fill")
                        }.font(.subheadline.weight(.semibold)).frame(maxWidth: .infinity, minHeight: 32)
                    }.buttonStyle(.glassProminent).disabled(saving || text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
                if let error { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }
                if receipt != .null {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Latest capture").font(.title3.weight(.semibold))
                        Button { selected = Record(receipt) } label: { captureRow(receipt) }.buttonStyle(.plain)
                    }
                }
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Needs review").font(.title3.weight(.semibold))
                        if !reviews.isEmpty { Text("\(reviews.count)").font(.subheadline).foregroundStyle(.secondary) }
                    }
                    ForEach(reviews) { capture in
                        Button { selected = capture } label: { captureRow(capture.value) }.buttonStyle(.plain)
                    }
                    if reviews.isEmpty { Text("No captures need review.").font(.subheadline).foregroundStyle(.secondary) }
                }
            }.padding(.horizontal, 24).padding(.vertical, 20).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.scrollDismissesKeyboard(.interactively).background(palette.canvas).tint(palette.accent).leadingPageTitle("Quick Add")
        .toolbar { ToolbarItem(placement: .topBarTrailing) { NavigationLink { QuickAddSettingsView() } label: { Label("Settings", systemImage: "gearshape") } } }
        .task(id: model.revision) { await load() }
        .refreshable { await load() }
        .sheet(item: $selected) { capture in NavigationStack { CaptureReviewView(capture: capture) } }
    }
    private func captureRow(_ capture: JSONValue) -> some View {
        HStack(spacing: 12) {
            IconTile(symbol: "text.viewfinder", tint: palette.accent)
            VStack(alignment: .leading, spacing: 5) {
                Text(capture["input"]["text"].text.isEmpty ? "Capture" : capture["input"]["text"].text).font(.body.weight(.medium)).lineLimit(3).multilineTextAlignment(.leading)
                Text(status(capture)).font(.caption).foregroundStyle(.secondary)
            }.frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right").font(.caption2.weight(.semibold)).foregroundStyle(.tertiary).accessibilityHidden(true)
        }.padding(16).background(palette.surface, in: .rect(cornerRadius: 22)).contentShape(.rect)
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
            Section("Captured text") {
                Text(current["input"]["text"].text)
                Text(current["status"].text.replacingOccurrences(of: "_", with: " ").capitalized).font(.caption).foregroundStyle(.secondary)
            }.surfaceRows()
            Section("Transaction") {
                Picker("Account", selection: binding("accountId")) { ForEach(model.accounts) { Text($0["name"].text).tag($0.id) } }
                Picker("Direction", selection: binding("direction")) { Text("Debit").tag("debit"); Text("Credit").tag("credit") }
                LabeledContent("Amount") { TextField("0.00", text: binding("amount")).keyboardType(.decimalPad).multilineTextAlignment(.trailing).monospacedDigit() }
                LabeledContent("Currency") { TextField(model.currency, text: binding("currency")).multilineTextAlignment(.trailing).textInputAutocapitalization(.characters).autocorrectionDisabled() }
                LabeledContent("Date") { TextField("YYYY-MM-DD", text: binding("date")).multilineTextAlignment(.trailing).keyboardType(.numbersAndPunctuation) }
                LabeledContent("Merchant") { TextField("Optional", text: binding("merchant")).multilineTextAlignment(.trailing) }
                LabeledContent("Reference") { TextField("Optional", text: binding("reference")).multilineTextAlignment(.trailing) }
            }.surfaceRows()
            if let error { Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }.surfaceRows() }
            Section {
                Button("Retry extraction", systemImage: "arrow.clockwise") { Task { await perform("/captures/" + capture.id + "/retry") } }.disabled(saving)
                Button("Dismiss capture", systemImage: "trash", role: .destructive) { Task { await perform("/capture-reviews/" + capture.id + "/dismiss") } }.disabled(saving)
            }.surfaceRows()
        }.themedForm().navigationTitle("Review capture").navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Close") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) { Button(saving ? "Saving…" : "Confirm") { Task { await resolve() } }.disabled(saving || fields.isEmpty) }
        }
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
            Section {
                LabeledContent("Provider") { TextField("Provider", text: binding("provider")).multilineTextAlignment(.trailing).textInputAutocapitalization(.never).autocorrectionDisabled() }
                LabeledContent("Model") { TextField("Model", text: binding("model")).multilineTextAlignment(.trailing).textInputAutocapitalization(.never).autocorrectionDisabled() }
                NavigationLink { AIProvidersView() } label: { Label("Provider credentials", systemImage: "key") }
            } header: { Text("Extraction") }.surfaceRows()
            Section("Defaults") {
                Picker("Default account", selection: binding("typedNoteAccountId")) {
                    Text("Choose an account").tag("")
                    ForEach(model.accounts) { Text($0["name"].text).tag($0.id) }
                }
                LabeledContent("Time zone") { TextField(model.settings["timezone"].text, text: binding("timezone")).multilineTextAlignment(.trailing).textInputAutocapitalization(.never).autocorrectionDisabled() }
            }.surfaceRows()
            if let error { Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }.surfaceRows() }
            Section { Button("Save settings") { Task {
                do { _ = try await model.mutate("/api/v1/quick-add/settings", method: "PUT", body: .object(settings)); error = nil }
                catch { self.error = error.localizedDescription }
            } }.disabled(settings.isEmpty) }.surfaceRows()
        }.themedForm().leadingPageTitle("Quick Add settings").task {
            do { settings = try await model.engine.request("/api/v1/quick-add/settings").fields } catch { self.error = error.localizedDescription }
        }
    }
}
