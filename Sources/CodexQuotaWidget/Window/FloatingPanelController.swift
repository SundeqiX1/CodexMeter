import AppKit
import SwiftUI

@MainActor
final class FloatingPanelController {
    private let panel: FloatingQuotaPanel
    private let defaultsKey = "floatingPanelVisible"

    init(store: QuotaStore) {
        let panel = FloatingQuotaPanel(
            contentRect: NSRect(x: 0, y: 0, width: 368, height: 360),
            styleMask: [.borderless, .nonactivatingPanel, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        self.panel = panel

        panel.level = .floating
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary]
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = false
        panel.hidesOnDeactivate = false
        panel.isMovableByWindowBackground = true
        panel.animationBehavior = .utilityWindow
        panel.becomesKeyOnlyIfNeeded = true

        panel.contentViewController = NSHostingController(
            rootView: FloatingPanelView(store: store) { [weak self] in
                self?.hide()
            }
        )

        let autosaveName = "CodexQuotaWidget.FloatingPanel"
        panel.setFrameAutosaveName(autosaveName)
        if !panel.setFrameUsingName(autosaveName) {
            positionAtTopRight()
        }
    }

    var isVisible: Bool {
        panel.isVisible
    }

    func show() {
        panel.orderFrontRegardless()
        UserDefaults.standard.set(true, forKey: defaultsKey)
    }

    func hide() {
        panel.orderOut(nil)
        UserDefaults.standard.set(false, forKey: defaultsKey)
    }

    func toggle() {
        isVisible ? hide() : show()
    }

    func restoreVisibility() {
        if UserDefaults.standard.object(forKey: defaultsKey) == nil {
            show()
        } else if UserDefaults.standard.bool(forKey: defaultsKey) {
            show()
        }
    }

    private func positionAtTopRight() {
        guard let visibleFrame = NSScreen.main?.visibleFrame else {
            panel.center()
            return
        }
        let panelSize = panel.frame.size
        let origin = NSPoint(
            x: visibleFrame.maxX - panelSize.width - 20,
            y: visibleFrame.maxY - panelSize.height - 20
        )
        panel.setFrameOrigin(origin)
    }
}

private final class FloatingQuotaPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}
