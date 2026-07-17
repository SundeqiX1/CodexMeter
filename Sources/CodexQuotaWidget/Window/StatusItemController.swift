import AppKit
import Combine
import SwiftUI

@MainActor
final class StatusItemController: NSObject {
    private let statusItem: NSStatusItem
    private let popover = NSPopover()
    private let store: QuotaStore
    private var cancellable: AnyCancellable?

    init(store: QuotaStore, floatingPanel: FloatingPanelController) {
        self.store = store
        self.statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        super.init()

        if let button = statusItem.button {
            let image = NSImage(
                systemSymbolName: "gauge.with.dots.needle.50percent",
                accessibilityDescription: "Codex 额度"
            )
            image?.isTemplate = true
            button.image = image
            button.imagePosition = .imageLeading
            button.font = .monospacedDigitSystemFont(ofSize: 12, weight: .semibold)
            button.target = self
            button.action = #selector(togglePopover(_:))
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }

        let contentView = MenuBarPopoverView(
            store: store,
            isFloatingVisible: { [weak floatingPanel] in
                floatingPanel?.isVisible ?? false
            },
            toggleFloating: { [weak floatingPanel] in
                floatingPanel?.toggle()
            }
        )
        let hostingController = NSHostingController(rootView: contentView)
        popover.contentViewController = hostingController
        popover.contentSize = NSSize(width: 370, height: 540)
        popover.behavior = .transient
        popover.animates = true

        updateButton()
        cancellable = store.objectWillChange.sink { [weak self] _ in
            DispatchQueue.main.async {
                self?.updateButton()
            }
        }
    }

    @objc
    private func togglePopover(_ sender: Any?) {
        guard let button = statusItem.button else { return }

        if let event = NSApp.currentEvent, event.type == .rightMouseUp {
            store.refresh()
            return
        }

        if popover.isShown {
            popover.performClose(sender)
        } else {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
        }
    }

    private func updateButton() {
        guard let button = statusItem.button else { return }
        button.title = " " + store.menuBarTitle
        if let window = store.criticalWindow {
            button.toolTip = "Codex \(window.windowLabel)剩余 \(window.remainingPercentText)；\(window.resetText)"
        } else if let error = store.errorMessage {
            button.toolTip = error
        } else {
            button.toolTip = "Codex 额度"
        }
    }
}
