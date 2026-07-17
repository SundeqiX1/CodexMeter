import AppKit
import SwiftUI

struct MenuBarPopoverView: View {
    @ObservedObject var store: QuotaStore
    let isFloatingVisible: () -> Bool
    let toggleFloating: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            QuotaDashboardView(store: store, showModelDetails: true)

            Divider()

            HStack(spacing: 12) {
                Button {
                    toggleFloating()
                } label: {
                    Label(
                        isFloatingVisible() ? "隐藏悬浮窗" : "显示悬浮窗",
                        systemImage: isFloatingVisible() ? "rectangle.slash" : "macwindow.on.rectangle"
                    )
                }
                .buttonStyle(.borderless)

                Spacer()

                Toggle(
                    "开机启动",
                    isOn: Binding(
                        get: { store.launchAtLoginEnabled },
                        set: { store.setLaunchAtLogin($0) }
                    )
                )
                .toggleStyle(.switch)
                .controlSize(.mini)
            }
            .font(.caption)
            .padding(.horizontal, 18)
            .padding(.vertical, 12)

            if let settingsMessage = store.settingsMessage {
                Text(settingsMessage)
                    .font(.caption2)
                    .foregroundStyle(.orange)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 18)
                    .padding(.bottom, 8)
            }

            Divider()

            HStack {
                Text("不读取或保存登录令牌")
                    .foregroundStyle(.tertiary)
                Spacer()
                Button("退出") {
                    NSApplication.shared.terminate(nil)
                }
                .buttonStyle(.borderless)
            }
            .font(.caption2)
            .padding(.horizontal, 18)
            .padding(.vertical, 10)
        }
        .frame(width: 370)
    }
}
