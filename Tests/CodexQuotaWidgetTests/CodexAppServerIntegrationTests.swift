import Foundation
import XCTest
@testable import CodexQuotaWidget

final class CodexAppServerIntegrationTests: XCTestCase {
    func testReadsLiveRateLimitsWhenEnabled() async throws {
        guard ProcessInfo.processInfo.environment["LIVE_CODEX_TEST"] == "1" else {
            throw XCTSkip("Set LIVE_CODEX_TEST=1 to query the signed-in local Codex account.")
        }

        let receivedSnapshot = expectation(description: "Received live Codex rate limits")
        let client = CodexAppServerClient()
        var capturedSnapshot: RateLimitsEnvelope?

        client.onSnapshot = { snapshot in
            capturedSnapshot = snapshot
            receivedSnapshot.fulfill()
        }
        client.onStateChange = { state in
            if case let .failed(message) = state {
                XCTFail(message)
            }
        }

        client.connect()
        await fulfillment(of: [receivedSnapshot], timeout: 35)
        client.disconnect()

        let snapshot = try XCTUnwrap(capturedSnapshot)
        XCTAssertFalse(snapshot.rateLimits.windows.isEmpty)
        XCTAssertTrue(snapshot.rateLimits.windows.allSatisfy { (0...100).contains($0.remainingPercent) })
    }
}
