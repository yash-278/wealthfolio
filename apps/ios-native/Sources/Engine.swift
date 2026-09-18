import Foundation

struct EngineError: LocalizedError {
    let message: String
    var errorDescription: String? { message }
}

final class Engine: Sendable {
    static let shared = Engine()
    private let queue = DispatchQueue(label: "wealthfolio.engine", qos: .userInitiated, attributes: .concurrent)
    private struct Reply: Decodable { let status: Int; let body: JSONValue }

    func open() async throws {
        let directory = try FileManager.default.url(for: .applicationSupportDirectory,
            in: .userDomainMask, appropriateFor: nil, create: true).appending(path: "WealthfolioNative")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        _ = try await call(directory.path, opening: true)
    }
    func request(_ path: String, method: String = "GET", body: JSONValue = .null) async throws -> JSONValue {
        let request = JSONValue.object(["method": .string(method), "path": .string(path), "body": body])
        let json = String(decoding: try JSONEncoder().encode(request), as: UTF8.self)
        return try await call(json)
    }
    func stream(body: JSONValue) -> AsyncThrowingStream<JSONValue, Error> {
        AsyncThrowingStream { continuation in
            let sink = NativeStreamSink(continuation)
            continuation.onTermination = { _ in sink.cancel() }
            queue.async {
                let context = Unmanaged.passRetained(sink).toOpaque()
                defer { Unmanaged<NativeStreamSink>.fromOpaque(context).release() }
                do {
                    let command = JSONValue.object(["method": .string("POST"), "path": .string("/api/v1/ai/chat/stream"), "body": body])
                    let json = String(decoding: try JSONEncoder().encode(command), as: UTF8.self)
                    let pointer = json.withCString { wf_native_stream($0, { event, context in
                        guard let event, let context else { return 0 }
                        return Unmanaged<NativeStreamSink>.fromOpaque(context).takeUnretainedValue().receive(event)
                    }, context) }
                    guard let pointer else { throw EngineError(message: "Assistant did not respond.") }
                    defer { wf_native_free(pointer) }
                    let reply = try JSONDecoder().decode(Reply.self, from: Data(String(cString: pointer).utf8))
                    if !(200..<300).contains(reply.status) {
                        throw EngineError(message: reply.body["message"].text.isEmpty ? "Assistant request failed." : reply.body["message"].text)
                    }
                    continuation.finish()
                } catch { continuation.finish(throwing: error) }
            }
        }
    }
    private func call(_ input: String, opening: Bool = false) async throws -> JSONValue {
        try await withCheckedThrowingContinuation { continuation in
            queue.async {
                do {
                    let pointer = input.withCString { opening ? wf_native_open($0) : wf_native_request($0) }
                    guard let pointer else { throw EngineError(message: "The local engine did not respond.") }
                    defer { wf_native_free(pointer) }
                    let data = Data(String(cString: pointer).utf8)
                    let reply = try JSONDecoder().decode(Reply.self, from: data)
                    guard (200..<300).contains(reply.status) else {
                        let message = [reply.body["message"].text, reply.body["error"].text, reply.body.text].first(where: { !$0.isEmpty }) ?? ""
                        throw EngineError(message: message.isEmpty ? "The request could not be completed (\(reply.status))." : message)
                    }
                    continuation.resume(returning: reply.body)
                } catch { continuation.resume(throwing: error) }
            }
        }
    }
}

private final class NativeStreamSink: @unchecked Sendable {
    private let continuation: AsyncThrowingStream<JSONValue, Error>.Continuation
    private let lock = NSLock()
    private var active = true
    init(_ continuation: AsyncThrowingStream<JSONValue, Error>.Continuation) { self.continuation = continuation }
    func cancel() { lock.lock(); active = false; lock.unlock() }
    func receive(_ pointer: UnsafePointer<CChar>) -> Int32 {
        lock.lock(); let allowed = active; lock.unlock()
        guard allowed else { return 0 }
        do {
            let event = try JSONDecoder().decode(JSONValue.self, from: Data(String(cString: pointer).utf8))
            continuation.yield(event)
            return 1
        } catch { continuation.finish(throwing: error); return 0 }
    }
}
