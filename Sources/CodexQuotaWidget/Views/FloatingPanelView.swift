import SwiftUI

struct FloatingPanelView: View {
    @ObservedObject var store: QuotaStore
    let onHide: () -> Void

    var body: some View {
        ZStack(alignment: .topTrailing) {
            QuotaDashboardView(store: store)

            Button(action: onHide) {
                Image(systemName: "xmark")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundStyle(.secondary)
                    .frame(width: 22, height: 22)
                    .background(.quaternary.opacity(0.8), in: Circle())
            }
            .buttonStyle(.plain)
            .padding(9)
            .help("隐藏悬浮窗")
        }
        .frame(width: 320)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .strokeBorder(.white.opacity(0.14), lineWidth: 1)
        }
        .shadow(color: .black.opacity(0.18), radius: 22, y: 9)
        .padding(24)
    }
}
