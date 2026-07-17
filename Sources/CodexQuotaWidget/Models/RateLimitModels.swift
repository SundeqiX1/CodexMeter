import Foundation

struct RateLimitsEnvelope: Decodable, Equatable {
    let rateLimits: RateLimitSnapshot
    let rateLimitsByLimitId: [String: RateLimitSnapshot]?
    let rateLimitResetCredits: RateLimitResetCreditsSummary?

    var orderedLimits: [RateLimitSnapshot] {
        guard let rateLimitsByLimitId, !rateLimitsByLimitId.isEmpty else {
            return [rateLimits]
        }

        return rateLimitsByLimitId.values.sorted { lhs, rhs in
            if lhs.limitId == "codex" { return true }
            if rhs.limitId == "codex" { return false }
            return lhs.displayName.localizedStandardCompare(rhs.displayName) == .orderedAscending
        }
    }
}

struct RateLimitSnapshot: Decodable, Equatable, Identifiable {
    let limitId: String?
    let limitName: String?
    let primary: RateLimitWindow?
    let secondary: RateLimitWindow?
    let credits: CreditsSnapshot?
    let individualLimit: SpendControlLimitSnapshot?
    let spendControlReached: Bool?
    let planType: String?
    let rateLimitReachedType: String?

    var id: String { limitId ?? limitName ?? "codex" }

    var displayName: String {
        if let limitName, !limitName.isEmpty { return limitName }
        return limitId == "codex" || limitId == nil ? "Codex" : limitId!
    }

    var windows: [RateLimitWindow] {
        [primary, secondary]
            .compactMap { $0 }
            .sorted {
                ($0.windowDurationMins ?? Int.max) < ($1.windowDurationMins ?? Int.max)
            }
    }

    var mostRelevantWindow: RateLimitWindow? {
        windows.first
    }
}

struct RateLimitWindow: Decodable, Equatable, Identifiable {
    let usedPercent: Double
    let windowDurationMins: Int?
    let resetsAt: TimeInterval?

    var id: String {
        "\(windowDurationMins ?? -1)-\(Int(resetsAt ?? 0))"
    }

    var remainingPercent: Double {
        min(100, max(0, 100 - usedPercent))
    }

    var remainingPercentText: String {
        "\(Int(remainingPercent.rounded()))%"
    }

    var windowLabel: String {
        guard let windowDurationMins else { return "额度窗口" }
        switch windowDurationMins {
        case 60:
            return "每小时"
        case 300:
            return "5 小时"
        case 1_440:
            return "每日"
        case 10_080:
            return "每周"
        case 43_200, 44_640:
            return "每月"
        default:
            if windowDurationMins.isMultiple(of: 1_440) {
                return "\(windowDurationMins / 1_440) 天"
            }
            if windowDurationMins.isMultiple(of: 60) {
                return "\(windowDurationMins / 60) 小时"
            }
            return "\(windowDurationMins) 分钟"
        }
    }

    var resetDate: Date? {
        resetsAt.map(Date.init(timeIntervalSince1970:))
    }

    var resetText: String {
        guard let resetDate else { return "重置时间未知" }
        return "\(resetDate.formatted(date: .abbreviated, time: .shortened)) 重置"
    }
}

struct CreditsSnapshot: Decodable, Equatable {
    let hasCredits: Bool
    let unlimited: Bool
    let balance: String?

    var displayBalance: String {
        if unlimited { return "无限" }
        return balance ?? "—"
    }
}

struct SpendControlLimitSnapshot: Decodable, Equatable {
    let limit: String
    let used: String
    let remainingPercent: Double
    let resetsAt: TimeInterval
}

struct RateLimitResetCreditsSummary: Decodable, Equatable {
    let availableCount: Int
    let credits: [RateLimitResetCredit]?
}

struct RateLimitResetCredit: Decodable, Equatable, Identifiable {
    let id: String
    let resetType: String
    let status: String
    let grantedAt: TimeInterval
    let expiresAt: TimeInterval?
    let title: String?
    let description: String?
}
