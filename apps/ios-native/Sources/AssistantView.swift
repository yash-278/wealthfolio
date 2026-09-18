import SwiftUI

struct AssistantView: View {
    @Environment(AppModel.self) private var model
    @State private var threads: [Record] = []
    @State private var creating = false
    var body: some View {
        List {
            ForEach(threads) { thread in
                NavigationLink(thread["title"].text.isEmpty ? "Conversation" : thread["title"].text) { ChatView(threadID: thread.id) }
            }
            if threads.isEmpty { ContentUnavailableView("Ask about your portfolio", systemImage: "sparkles", description: Text("Start a conversation with your configured AI provider.")) }
            NavigationLink("AI provider settings") { AIProvidersView() }
        }.navigationTitle("Assistant")
        .toolbar { NavigationLink { ChatView() } label: { Label("New conversation", systemImage: "square.and.pencil") } }
        .task(id: model.revision) {
            do { threads = try await model.engine.request("/api/v1/ai/threads")["threads"].values.map(Record.init) }
            catch { model.error = error.localizedDescription }
        }
    }
}
struct ChatView: View {
    @Environment(AppModel.self) private var model
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
                    VStack(alignment: .leading, spacing: 6) {
                        Text(message["role"].text.capitalized).font(.caption).foregroundStyle(.secondary)
                        ForEach(Array(message["content"]["parts"].values.enumerated()), id: \.offset) { _, part in
                            if !part["content"].text.isEmpty { Text(.init(part["content"].text)).textSelection(.enabled) }
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                if sending {
                    Text(pending)
                    if response.isEmpty { ProgressView("Thinking…") }
                    else { Text(.init(response)).textSelection(.enabled) }
                }
                if let error { Text(error).foregroundStyle(.red) }
            }.padding()
        }.navigationTitle("Assistant")
        .safeAreaInset(edge: .bottom) {
            HStack(alignment: .bottom) {
                TextField("Ask about your portfolio", text: $input, axis: .vertical).lineLimit(1...6)
                Button("Send", systemImage: "arrow.up.circle.fill") { Task { await send() } }.labelStyle(.iconOnly).disabled(sending || input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }.padding().background(.bar)
        }.task { await load() }
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
