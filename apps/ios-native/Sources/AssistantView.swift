import SwiftUI

struct AssistantView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State private var threads: [Record] = []
    var body: some View {
        List {
            ForEach(threads) { thread in
                NavigationLink { ChatView(threadID: thread.id) } label: {
                    HStack(spacing: 12) {
                        IconTile(symbol: thread["isPinned"].flag ? "pin" : "bubble.left", tint: palette.accent)
                        VStack(alignment: .leading, spacing: 5) {
                            Text(thread["title"].text.isEmpty ? "Conversation" : thread["title"].text).font(.body.weight(.medium)).lineLimit(2)
                            if let updated = parseActivityDate(thread["updatedAt"].text) {
                                Text(updated.formatted(.relative(presentation: .named))).font(.caption).foregroundStyle(.secondary)
                            }
                        }
                    }.padding(.vertical, 6)
                }.listRowBackground(Color.clear).listRowSeparator(.hidden)
                .swipeActions { Button("Delete", role: .destructive) { Task { await delete(thread) } } }
            }
            if threads.isEmpty {
                ContentUnavailableView {
                    Label("Ask about your portfolio", systemImage: "sparkles")
                } description: { Text("Start a conversation with your configured AI provider.") } actions: {
                    NavigationLink("New conversation") { ChatView() }.buttonStyle(.glass)
                }.listRowBackground(Color.clear).listRowSeparator(.hidden)
            }
        }.listStyle(.plain).themedForm().leadingPageTitle("Assistant")
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) { NavigationLink { AIProvidersView() } label: { Label("AI providers", systemImage: "slider.horizontal.3") } }
            ToolbarItem(placement: .topBarTrailing) { NavigationLink { ChatView() } label: { Label("New conversation", systemImage: "square.and.pencil") } }
        }
        .task(id: model.revision) { await load() }.refreshable { await load() }
    }
    private func load() async {
        do { threads = try await model.engine.request("/api/v1/ai/threads")["threads"].values.map(Record.init) }
        catch { model.error = error.localizedDescription }
    }
    private func delete(_ thread: Record) async {
        do { _ = try await model.engine.request("/api/v1/ai/threads/" + thread.id, method: "DELETE"); await load() }
        catch { model.error = error.localizedDescription }
    }
}
struct ChatView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    private var palette: InsightPalette { InsightPalette(scheme: scheme) }
    @State var threadID: String?
    @State private var messages: [Record] = []
    @State private var input = ""
    @State private var sending = false
    @State private var pending = ""
    @State private var response = ""
    @State private var error: String?
    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 20) {
                ForEach(messages) { message in
                    let text = message["content"]["parts"].values.map { $0["content"].text }.filter { !$0.isEmpty }.joined(separator: "\n\n")
                    if !text.isEmpty { bubble(text, user: message["role"].text == "user") }
                }
                if sending {
                    bubble(pending, user: true)
                    if response.isEmpty { ProgressView("Thinking…").font(.subheadline) }
                    else { bubble(response, user: false) }
                }
                if messages.isEmpty && !sending && error == nil {
                    ContentUnavailableView("Ask about your portfolio", systemImage: "sparkles", description: Text("Answers use the portfolio stored on this device."))
                        .frame(maxWidth: .infinity).padding(.top, 60)
                }
                if let error { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red) }
            }.padding(.horizontal, 20).padding(.vertical, 16).frame(maxWidth: 720).frame(maxWidth: .infinity)
        }.defaultScrollAnchor(.bottom).scrollDismissesKeyboard(.interactively)
        .background(palette.canvas).tint(palette.accent).leadingPageTitle("Assistant")
        .safeAreaInset(edge: .bottom) {
            GlassEffectContainer(spacing: 8) {
                HStack(alignment: .bottom, spacing: 8) {
                    TextField("Ask about your portfolio", text: $input, axis: .vertical).lineLimit(1...6)
                        .padding(.horizontal, 16).padding(.vertical, 12).frame(minHeight: 44)
                        .glassEffect(.regular, in: .rect(cornerRadius: 22))
                    Button { Task { await send() } } label: {
                        Image(systemName: "arrow.up").font(.body.weight(.semibold)).frame(width: 44, height: 44)
                            .contentShape(.circle).glassEffect(.regular.tint(palette.accent.opacity(0.35)).interactive(), in: .circle)
                    }.buttonStyle(.plain).accessibilityLabel("Send")
                        .disabled(sending || input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }.padding(.horizontal, 16).padding(.vertical, 8)
        }.task { await load() }
    }
    // The role stays readable without color: user messages are trailing and labelled for VoiceOver.
    private func bubble(_ text: String, user: Bool) -> some View {
        Text(.init(text)).textSelection(.enabled)
            .padding(user ? 14 : 0)
            .background(user ? palette.surface : .clear, in: .rect(cornerRadius: 20))
            .frame(maxWidth: .infinity, alignment: user ? .trailing : .leading)
            .padding(.leading, user ? 48 : 0)
            .accessibilityLabel((user ? "You: " : "Assistant: ") + text)
    }
    private func load() async {
        guard let threadID else { return }
        do { messages = try await model.engine.request("/api/v1/ai/threads/" + threadID + "/messages").values.map(Record.init) }
        catch { self.error = error.localizedDescription }
    }
    private func send() async {
        sending = true; pending = input; input = ""; response = ""; error = nil
        defer { sending = false }
        var body: [String: JSONValue] = ["content": .string(pending)]
        if let threadID { body["threadId"] = .string(threadID) }
        do {
            for try await event in model.engine.stream(body: .object(body)) {
                if !event["threadId"].text.isEmpty { threadID = event["threadId"].text }
                if event["type"].text == "textDelta" { response += event["delta"].text }
                if event["type"].text == "error" { error = event["message"].text }
            }
            await load(); await model.refresh()
        } catch { self.error = error.localizedDescription; input = pending }
    }
}
