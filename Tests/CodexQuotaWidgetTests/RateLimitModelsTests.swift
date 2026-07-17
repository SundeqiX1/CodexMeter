import Foundation
import XCTest
@testable import CodexQuotaWidget

final class RateLimitModelsTests: XCTestCase {
    func testDecodesCurrentRateLimitShapeAndCalculatesRemaining() throws {
        let data = Data(
            #"""
            {
              "rateLimits": {
                "limitId": "codex",
                "limitName": null,
                "primary": {
                  "usedPercent": 46,
                  "windowDurationMins": 10080,
                  "resetsAt": 1784785660
                },
                "secondary": null,
                "credits": {
                  "hasCredits": false,
                  "unlimited": false,
                  "balance": "0"
                },
                "individualLimit": null,
                "spendControlReached": false,
                "planType": "prolite",
                "rateLimitReachedType": null
              },
              "rateLimitsByLimitId": null,
              "rateLimitResetCredits": {
                "availableCount": 1,
                "credits": null
              }
            }
            """#.utf8
        )

        let envelope = try JSONDecoder().decode(RateLimitsEnvelope.self, from: data)
        let window = try XCTUnwrap(envelope.rateLimits.primary)

        XCTAssertEqual(window.remainingPercent, 54)
        XCTAssertEqual(window.remainingPercentText, "54%")
        XCTAssertEqual(window.windowLabel, "每周")
        XCTAssertEqual(envelope.rateLimitResetCredits?.availableCount, 1)
        XCTAssertEqual(envelope.rateLimits.credits?.displayBalance, "0")
    }

    func testWindowLabelsAreBasedOnDurationInsteadOfPrimaryPosition() {
        XCTAssertEqual(makeWindow(minutes: 300).windowLabel, "5 小时")
        XCTAssertEqual(makeWindow(minutes: 10_080).windowLabel, "每周")
        XCTAssertEqual(makeWindow(minutes: 2_880).windowLabel, "2 天")
        XCTAssertEqual(makeWindow(minutes: 90).windowLabel, "90 分钟")
    }

    func testRemainingIsClamped() {
        XCTAssertEqual(makeWindow(used: -4).remainingPercent, 100)
        XCTAssertEqual(makeWindow(used: 104).remainingPercent, 0)
    }

    private func makeWindow(used: Double = 25, minutes: Int = 300) -> RateLimitWindow {
        RateLimitWindow(usedPercent: used, windowDurationMins: minutes, resetsAt: nil)
    }
}
