import Foundation

enum JSONValue: Codable, Sendable, Equatable {
    case object([String: JSONValue]), array([JSONValue]), string(String), number(Decimal), bool(Bool), null
    init(from decoder: Decoder) throws {
        let box = try decoder.singleValueContainer()
        if box.decodeNil() { self = .null }
        else if let value = try? box.decode(Bool.self) { self = .bool(value) }
        else if let value = try? box.decode(String.self) { self = .string(value) }
        else if let value = try? box.decode(Decimal.self) { self = .number(value) }
        else if let value = try? box.decode([JSONValue].self) { self = .array(value) }
        else { self = .object(try box.decode([String: JSONValue].self)) }
    }
    func encode(to encoder: Encoder) throws {
        var box = encoder.singleValueContainer()
        switch self {
        case .object(let value): try box.encode(value)
        case .array(let value): try box.encode(value)
        case .string(let value): try box.encode(value)
        case .number(let value): try box.encode(value)
        case .bool(let value): try box.encode(value)
        case .null: try box.encodeNil()
        }
    }
    subscript(key: String) -> JSONValue {
        guard case .object(let fields) = self else { return .null }
        return fields[key] ?? .null
    }
    var text: String {
        switch self { case .string(let s): return s; case .number(let n): return NSDecimalNumber(decimal: n).stringValue; default: return "" }
    }
    var values: [JSONValue] { if case .array(let a) = self { return a }; return [] }
    var flag: Bool { self == .bool(true) }
    var decimal: Decimal? { Decimal(string: text, locale: Locale(identifier: "en_US_POSIX")) }
    var fields: [String: JSONValue] { if case .object(let fields) = self { return fields }; return [:] }
    static func strings(_ fields: [String: String]) -> JSONValue { .object(fields.mapValues(JSONValue.string)) }
}

struct Record: Identifiable, Hashable {
    let id: String
    let value: JSONValue
    init(_ value: JSONValue) { self.value = value; id = value["id"].text }
    static func == (lhs: Record, rhs: Record) -> Bool { lhs.id == rhs.id && lhs.value == rhs.value }
    func hash(into hasher: inout Hasher) { hasher.combine(id) }
    subscript(_ key: String) -> JSONValue { value[key] }
}

func money(_ value: JSONValue, currency: String, hidden: Bool = false) -> String {
    if hidden { return "••••" }
    guard let amount = value.decimal, !currency.isEmpty else { return "—" }
    return amount.formatted(.currency(code: currency))
}

func parseActivityDate(_ text: String) -> Date? {
    let formatter = ISO8601DateFormatter()
    if let date = formatter.date(from: text) { return date }
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter.date(from: text)
}
