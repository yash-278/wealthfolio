import XCTest
@testable import WealthfolioNative

final class JSONValueTests: XCTestCase {
    func testFinancialDecimalsRoundTripWithoutFloatingPointRounding() throws {
        let original = Data(#"{"amount":123456789012345.6789,"quantity":"0.000000000123456789","missing":null,"enabled":true}"#.utf8)
        let value = try JSONDecoder().decode(JSONValue.self, from: original)
        XCTAssertEqual(value["amount"].decimal, Decimal(string: "123456789012345.6789"))
        XCTAssertEqual(value["quantity"].decimal, Decimal(string: "0.000000000123456789"))
        XCTAssertNil(value["missing"].decimal)
        XCTAssertTrue(value["enabled"].flag)
        let roundTrip = try JSONDecoder().decode(JSONValue.self, from: JSONEncoder().encode(value))
        XCTAssertEqual(value, roundTrip)
    }

    func testActivityDatesPreserveOffsetAndFractionalSeconds() {
        XCTAssertEqual(parseActivityDate("2026-09-13T17:30:00+05:30"), parseActivityDate("2026-09-13T12:00:00Z"))
        XCTAssertNotNil(parseActivityDate("2026-09-13T12:00:00.123Z"))
        XCTAssertNil(parseActivityDate("invalid"))
    }

    func testMissingFinancialValuesAreNotPresentedAsZero() {
        XCTAssertEqual(money(.null, currency: "USD"), "—")
        XCTAssertEqual(money(.number(123), currency: "USD", hidden: true), "••••")
    }
}
