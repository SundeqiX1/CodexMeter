import Combine
import Foundation
import ServiceManagement

@MainActor
final class QuotaStore: ObservableObject {
    @Published private(set) var snapshot: RateLimitsEnvelope?
    @Published private(set) var connectionState: CodexAppServerClient.ConnectionState = .disconnected
    @Published private(set) var lastUpdated: Date?
    @Published private(set) var isRefreshing = false
    @Published private(set) var launchAtLoginEnabled = false
    @Published private(set) var settingsMessage: String?

    private let client: CodexAppServerClient
    private var refreshTimer: Timer?
    private var reconnectWorkItem: DispatchWorkItem?

    init(client: CodexAppServerClient = CodexAppServerClient()) {
        self.client = client
        self.launchAtLoginEnabled = SMAppService.mainApp.status == .enabled

        client.onSnapshot = { [weak self] snapshot in
            guard let self else { return }
            self.snapshot = snapshot
            self.lastUpdated = Date()
            self.isRefreshing = false
            self.settingsMessage = nil
        }

        client.onStateChange = { [weak self] state in
            guard let self else { return }
            self.connectionState = state

            switch state {
            case .connecting:
                self.isRefreshing = true
            case .connected:
                break
            case .disconnected:
                self.isRefreshing = false
            case .failed:
                self.isRefreshing = false
                self.scheduleReconnect()
            }
        }
    }

    func start() {
        reconnectWorkItem?.cancel()
        isRefreshing = true
        client.connect()

        refreshTimer?.invalidate()
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 300, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.refresh()
            }
        }
        refreshTimer?.tolerance = 20
    }

    func stop() {
        reconnectWorkItem?.cancel()
        refreshTimer?.invalidate()
        refreshTimer = nil
        client.disconnect()
    }

    func refresh() {
        reconnectWorkItem?.cancel()
        isRefreshing = true
        client.refresh()
    }

    func reconnect() {
        client.disconnect()
        let workItem = DispatchWorkItem { [weak self] in
            Task { @MainActor in
                guard let self else { return }
                self.isRefreshing = true
                self.client.connect()
            }
        }
        reconnectWorkItem = workItem
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5, execute: workItem)
    }

    func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
            launchAtLoginEnabled = SMAppService.mainApp.status == .enabled
            settingsMessage = launchAtLoginEnabled == enabled
                ? nil
                : "系统尚未确认开机启动设置。"
        } catch {
            launchAtLoginEnabled = SMAppService.mainApp.status == .enabled
            settingsMessage = "开机启动设置失败：\(error.localizedDescription)"
        }
    }

    var mainLimit: RateLimitSnapshot? {
        snapshot?.rateLimits
    }

    var criticalWindow: RateLimitWindow? {
        mainLimit?.windows.min { lhs, rhs in
            lhs.remainingPercent < rhs.remainingPercent
        }
    }

    var menuBarTitle: String {
        if let criticalWindow {
            return criticalWindow.remainingPercentText
        }
        switch connectionState {
        case .connecting:
            return "…"
        case .failed:
            return "!"
        default:
            return "—"
        }
    }

    var planDisplayName: String? {
        guard let plan = mainLimit?.planType else { return nil }
        let names = [
            "free": "Free",
            "go": "Go",
            "plus": "Plus",
            "pro": "Pro",
            "prolite": "Pro Lite",
            "team": "Team",
            "business": "Business",
            "self_serve_business_usage_based": "Business",
            "enterprise": "Enterprise",
            "enterprise_cbp_usage_based": "Enterprise",
            "edu": "Edu"
        ]
        return names[plan] ?? plan
    }

    var errorMessage: String? {
        guard case let .failed(message) = connectionState else { return nil }
        return message
    }

    var connectionLabel: String {
        switch connectionState {
        case .disconnected:
            return "未连接"
        case .connecting:
            return "连接中"
        case .connected:
            return "已连接"
        case .failed:
            return "连接异常"
        }
    }

    private func scheduleReconnect() {
        reconnectWorkItem?.cancel()
        let workItem = DispatchWorkItem { [weak self] in
            Task { @MainActor in
                self?.refresh()
            }
        }
        reconnectWorkItem = workItem
        DispatchQueue.main.asyncAfter(deadline: .now() + 15, execute: workItem)
    }
}
