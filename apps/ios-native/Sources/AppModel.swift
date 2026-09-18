import SwiftUI
import Observation

@MainActor @Observable final class AppModel {
    let engine = Engine.shared
    var ready = false
    var loaded = false
    var startupError: String?
    var error: String?
    var accounts: [Record] = []
    var valuation: JSONValue = .null
    var history: [JSONValue] = []
    var dashboardPoints: [DashboardPoint] = []
    var holdings: [Record] = []
    var settings: JSONValue = .null
    var sync: JSONValue = .null
    var syncing = false
    var syncError: String?
    var revision = 0
    var hideBalances = UserDefaults.standard.bool(forKey: "hideBalances") {
        didSet { UserDefaults.standard.set(hideBalances, forKey: "hideBalances") }
    }
    var currency: String { settings["baseCurrency"].text }
    var connected: Bool { sync["connection"] != .null }
    private var refreshing = false
    private var starting = false
    private let all: JSONValue = .object(["filter": .object(["type": .string("all")]), "includeAccounts": .bool(true)])

    func start() async {
        guard !ready && !starting else { return }
        starting = true
        defer { starting = false }
        do { try await engine.open(); ready = true; await refresh() }
        catch { startupError = error.localizedDescription }
    }
    func refresh() async {
        guard ready, !refreshing else { return }
        refreshing = true
        defer { refreshing = false }
        do {
            accounts = try await engine.request("/api/v1/accounts").values.map(Record.init)
            settings = try await engine.request("/api/v1/settings")
            valuation = try await engine.request("/api/v1/valuations/current/query", method: "POST", body: all)
            holdings = try await engine.request("/api/v1/holdings/list/query", method: "POST", body: all).values.map(Record.init)
            history = try await engine.request("/api/v1/valuations/history/query", method: "POST", body: all).values
            let formatter = DateFormatter()
            formatter.locale = Locale(identifier: "en_US_POSIX")
            formatter.calendar = Calendar(identifier: .gregorian)
            formatter.timeZone = TimeZone(identifier: settings["timezone"].text) ?? .current
            formatter.dateFormat = "yyyy-MM-dd"
            dashboardPoints = history.compactMap { row in
                guard let date = formatter.date(from: row["valuationDate"].text), let value = row["totalValueBase"].decimal else { return nil }
                return DashboardPoint(date: date, value: value)
            }.sorted { $0.date < $1.date }
            sync = try await engine.request("/native/sync/status")
            loaded = true
            revision += 1
        } catch { self.error = error.localizedDescription }
    }
    func synchronize() async {
        guard ready, !syncing else { return }
        syncing = true
        defer { syncing = false }
        do {
            sync = try await engine.request("/native/sync/status")
            if connected && !sync["connection"]["paused"].flag {
                sync = try await engine.request("/native/sync/run", method: "POST")
                syncError = nil
                await refresh()
            }
        } catch { self.syncError = error.localizedDescription }
    }
    func mutate(_ path: String, method: String = "POST", body: JSONValue = .null) async throws -> JSONValue {
        let result = try await engine.request(path, method: method, body: body)
        await refresh()
        Task { await synchronize() }
        return result
    }
}
