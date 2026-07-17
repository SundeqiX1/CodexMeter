import SwiftUI

struct QuotaDashboardView: View {
    @ObservedObject var store: QuotaStore
    var showModelDetails = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            header

            if let mainLimit = store.mainLimit {
                if mainLimit.windows.isEmpty {
                    ContentUnavailableView(
                        "暂无额度窗口",
                        systemImage: "gauge.with.dots.needle.0percent",
                        description: Text("当前账户没有返回可展示的额度窗口。")
                    )
                    .frame(minHeight: 120)
                } else {
                    VStack(spacing: 13) {
                        ForEach(mainLimit.windows) { window in
                            QuotaWindowRow(window: window)
                        }
                    }
                }

                metadata(limit: mainLimit)

                if showModelDetails {
                    modelSpecificLimits
                }
            } else if store.isRefreshing {
                loadingState
            } else {
                emptyState
            }

            if let errorMessage = store.errorMessage {
                errorBanner(errorMessage)
            }

            footer
        }
        .padding(18)
    }

    private var header: some View {
        HStack(spacing: 11) {
            ZStack {
                RoundedRectangle(cornerRadius: 11, style: .continuous)
                    .fill(.tint.opacity(0.14))
                Image(systemName: "gauge.with.dots.needle.67percent")
                    .font(.system(size: 20, weight: .semibold))
                    .foregroundStyle(.tint)
            }
            .frame(width: 42, height: 42)

            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 7) {
                    Text("Codex 额度")
                        .font(.headline)
                    if let plan = store.planDisplayName {
                        Text(plan)
                            .font(.caption2.weight(.semibold))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 7)
                            .padding(.vertical, 3)
                            .background(.quaternary, in: Capsule())
                    }
                }
                HStack(spacing: 5) {
                    Circle()
                        .fill(connectionColor)
                        .frame(width: 6, height: 6)
                    Text(store.connectionLabel)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer(minLength: 8)

            Button {
                store.refresh()
            } label: {
                Image(systemName: "arrow.clockwise")
                    .font(.system(size: 13, weight: .semibold))
                    .rotationEffect(store.isRefreshing ? .degrees(360) : .zero)
                    .animation(
                        store.isRefreshing
                            ? .linear(duration: 0.9).repeatForever(autoreverses: false)
                            : .default,
                        value: store.isRefreshing
                    )
            }
            .buttonStyle(.plain)
            .foregroundStyle(.secondary)
            .padding(8)
            .background(.quaternary.opacity(0.7), in: Circle())
            .help("立即刷新")
            .disabled(store.isRefreshing)
        }
    }

    private var loadingState: some View {
        HStack(spacing: 12) {
            ProgressView()
                .controlSize(.small)
            VStack(alignment: .leading, spacing: 3) {
                Text("正在读取额度")
                    .font(.subheadline.weight(.semibold))
                Text("使用本机 Codex 登录状态连接 App Server")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, minHeight: 110, alignment: .center)
    }

    private var emptyState: some View {
        ContentUnavailableView(
            "尚无额度数据",
            systemImage: "bolt.horizontal.circle",
            description: Text("点击刷新重新连接本机 Codex。")
        )
        .frame(minHeight: 140)
    }

    @ViewBuilder
    private func metadata(limit: RateLimitSnapshot) -> some View {
        let resetCount = store.snapshot?.rateLimitResetCredits?.availableCount
        if limit.credits != nil || resetCount != nil || limit.individualLimit != nil {
            HStack(spacing: 8) {
                if let credits = limit.credits {
                    MetricPill(
                        icon: "creditcard",
                        label: "积分",
                        value: credits.displayBalance
                    )
                }

                if let resetCount {
                    MetricPill(
                        icon: "arrow.counterclockwise.circle",
                        label: "重置券",
                        value: "×\(resetCount)"
                    )
                }

                if let individualLimit = limit.individualLimit {
                    MetricPill(
                        icon: "person.crop.circle",
                        label: "月额度",
                        value: "\(Int(individualLimit.remainingPercent.rounded()))%"
                    )
                }
            }
        }
    }

    @ViewBuilder
    private var modelSpecificLimits: some View {
        if let limits = store.snapshot?.orderedLimits.filter({ $0.limitId != "codex" }), !limits.isEmpty {
            VStack(alignment: .leading, spacing: 10) {
                Divider()
                Text("模型额度")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                ForEach(limits) { limit in
                    VStack(alignment: .leading, spacing: 8) {
                        Text(limit.displayName)
                            .font(.subheadline.weight(.medium))
                        ForEach(limit.windows) { window in
                            CompactQuotaRow(window: window)
                        }
                    }
                }
            }
        }
    }

    private func errorBanner(_ message: String) -> some View {
        HStack(alignment: .top, spacing: 9) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.orange)
            Text(message)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
            Spacer(minLength: 4)
            Button("重连") {
                store.reconnect()
            }
            .buttonStyle(.borderless)
            .font(.caption.weight(.semibold))
        }
        .padding(10)
        .background(.orange.opacity(0.09), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private var footer: some View {
        HStack {
            if let lastUpdated = store.lastUpdated {
                Text("更新于 \(lastUpdated.formatted(date: .omitted, time: .shortened))")
            } else {
                Text("每 5 分钟自动刷新")
            }
            Spacer()
            Text("本地连接")
        }
        .font(.caption2)
        .foregroundStyle(.tertiary)
    }

    private var connectionColor: Color {
        switch store.connectionState {
        case .connected:
            return .green
        case .connecting:
            return .yellow
        case .failed:
            return .orange
        case .disconnected:
            return .secondary
        }
    }
}

private struct QuotaWindowRow: View {
    let window: RateLimitWindow

    var body: some View {
        VStack(spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(window.windowLabel)
                        .font(.subheadline.weight(.semibold))
                    Text(window.resetText)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text(window.remainingPercentText)
                    .font(.system(.title2, design: .rounded, weight: .bold))
                    .monospacedDigit()
                    .foregroundStyle(severityColor)
            }

            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(.quaternary)
                    Capsule()
                        .fill(severityColor.gradient)
                        .frame(width: max(5, proxy.size.width * window.remainingPercent / 100))
                }
            }
            .frame(height: 8)
            .accessibilityLabel("\(window.windowLabel)剩余")
            .accessibilityValue(window.remainingPercentText)
        }
    }

    private var severityColor: Color {
        QuotaPalette.color(forRemaining: window.remainingPercent)
    }
}

private struct CompactQuotaRow: View {
    let window: RateLimitWindow

    var body: some View {
        HStack(spacing: 9) {
            Text(window.windowLabel)
                .foregroundStyle(.secondary)
            ProgressView(value: window.remainingPercent, total: 100)
                .tint(QuotaPalette.color(forRemaining: window.remainingPercent))
            Text(window.remainingPercentText)
                .font(.caption.monospacedDigit().weight(.semibold))
                .frame(width: 34, alignment: .trailing)
        }
        .font(.caption)
    }
}

private struct MetricPill: View {
    let icon: String
    let label: String
    let value: String

    var body: some View {
        HStack(spacing: 5) {
            Image(systemName: icon)
                .foregroundStyle(.secondary)
            Text(label)
                .foregroundStyle(.secondary)
            Text(value)
                .fontWeight(.semibold)
                .monospacedDigit()
        }
        .font(.caption)
        .lineLimit(1)
        .padding(.horizontal, 9)
        .padding(.vertical, 7)
        .background(.quaternary.opacity(0.65), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
    }
}

enum QuotaPalette {
    static func color(forRemaining remaining: Double) -> Color {
        switch remaining {
        case ..<10:
            return .red
        case ..<25:
            return .orange
        case ..<50:
            return .yellow
        default:
            return .green
        }
    }
}
