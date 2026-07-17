import AppKit

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private let store = QuotaStore()
    private var floatingPanelController: FloatingPanelController?
    private var statusItemController: StatusItemController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApplication.shared.setActivationPolicy(.accessory)

        let floatingPanelController = FloatingPanelController(store: store)
        self.floatingPanelController = floatingPanelController
        self.statusItemController = StatusItemController(
            store: store,
            floatingPanel: floatingPanelController
        )

        floatingPanelController.restoreVisibility()
        store.start()
    }

    func applicationWillTerminate(_ notification: Notification) {
        store.stop()
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }
}
