import XCTest

// Walks the secondary screens on a fresh install and attaches a screenshot of each.
final class ScreenTourTests: XCTestCase {
    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = true
        app.launch()
    }

    func testTour() {
        let start = app.buttons["Continue"]
        if start.waitForExistence(timeout: 20) { capture("Setup"); start.tap() }
        XCTAssertTrue(app.tabBars.buttons["More"].waitForExistence(timeout: 30))

        app.tabBars.buttons["Assistant"].tap(); capture("Assistant")
        app.buttons["New conversation"].firstMatch.tap(); capture("Chat"); back()
        app.buttons["AI providers"].firstMatch.tap(); capture("AI providers")
        let provider = app.cells.firstMatch
        if provider.waitForExistence(timeout: 5) { provider.tap(); capture("AI provider"); back() }
        back()

        app.tabBars.buttons["More"].tap(); capture("More")
        open("Quick Add"); capture("Quick Add")
        app.buttons["Settings"].firstMatch.tap(); capture("Quick Add settings"); back(); back()
        open("Your server"); capture("Your server"); back()
        open("Settings"); capture("Settings")
        open("Export CSV"); capture("Export"); back(); back()

        app.tabBars.buttons["Dashboard"].tap()
        app.buttons["Add account"].firstMatch.tap(); capture("New account")
        let name = app.textFields["Account name"]
        XCTAssertTrue(name.waitForExistence(timeout: 5))
        name.tap(); name.typeText("Tour account")
        app.buttons["Save"].tap()
        let row = app.buttons.containing(NSPredicate(format: "label CONTAINS 'Tour account'")).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 15))
        row.tap(); capture("Account detail")
    }

    private func open(_ title: String) {
        let target = app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", title)).firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 5), title)
        target.tap()
    }
    private func back() { app.navigationBars.buttons.element(boundBy: 0).tap() }
    private func capture(_ name: String) {
        Thread.sleep(forTimeInterval: 1)
        let attachment = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        attachment.name = name; attachment.lifetime = .keepAlways
        add(attachment)
    }
}
