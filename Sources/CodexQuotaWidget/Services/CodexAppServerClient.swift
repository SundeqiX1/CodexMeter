import Foundation

final class CodexAppServerClient {
    enum ConnectionState: Equatable {
        case disconnected
        case connecting
        case connected
        case failed(String)
    }

    var onSnapshot: ((RateLimitsEnvelope) -> Void)?
    var onStateChange: ((ConnectionState) -> Void)?

    private let ioQueue = DispatchQueue(label: "io.github.changzhengithub.codexquotatool.app-server")
    private var process: Process?
    private var inputHandle: FileHandle?
    private var outputPipe: Pipe?
    private var errorPipe: Pipe?
    private var outputBuffer = Data()
    private var errorTail = ""
    private var nextRequestID = 10
    private var initializeRequestID = 1
    private var pendingRateLimitRequestID: Int?
    private var didInitialize = false
    private var isStopping = false

    func connect() {
        ioQueue.async { [weak self] in
            self?.connectLocked()
        }
    }

    func refresh() {
        ioQueue.async { [weak self] in
            guard let self else { return }
            if self.process?.isRunning == true, self.didInitialize {
                self.requestRateLimitsLocked()
            } else {
                self.connectLocked()
            }
        }
    }

    func disconnect() {
        ioQueue.async { [weak self] in
            guard let self else { return }
            self.isStopping = true
            self.outputPipe?.fileHandleForReading.readabilityHandler = nil
            self.errorPipe?.fileHandleForReading.readabilityHandler = nil
            self.inputHandle?.closeFile()
            if self.process?.isRunning == true {
                self.process?.terminate()
            }
            self.clearProcessLocked()
            self.emitState(.disconnected)
        }
    }

    private func connectLocked() {
        guard process?.isRunning != true else { return }
        guard let executableURL = Self.locateCodexExecutable() else {
            emitState(.failed("找不到 Codex。请先安装或打开 Codex/ChatGPT 桌面应用。"))
            return
        }

        isStopping = false
        didInitialize = false
        outputBuffer.removeAll(keepingCapacity: true)
        errorTail = ""
        pendingRateLimitRequestID = nil
        emitState(.connecting)

        let outputPipe = Pipe()
        let errorPipe = Pipe()
        let inputPipe = Pipe()
        let process = Process()
        process.executableURL = executableURL
        process.arguments = ["app-server", "--stdio"]
        process.standardInput = inputPipe
        process.standardOutput = outputPipe
        process.standardError = errorPipe

        outputPipe.fileHandleForReading.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            guard !data.isEmpty else { return }
            self?.ioQueue.async { [weak self] in
                self?.consumeOutputLocked(data)
            }
        }

        errorPipe.fileHandleForReading.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            guard !data.isEmpty, let text = String(data: data, encoding: .utf8) else { return }
            self?.ioQueue.async { [weak self] in
                self?.appendErrorTailLocked(text)
            }
        }

        process.terminationHandler = { [weak self] terminatedProcess in
            self?.ioQueue.async { [weak self] in
                self?.handleTerminationLocked(status: terminatedProcess.terminationStatus)
            }
        }

        self.process = process
        self.inputHandle = inputPipe.fileHandleForWriting
        self.outputPipe = outputPipe
        self.errorPipe = errorPipe

        do {
            try process.run()
            try sendLocked([
                "method": "initialize",
                "id": initializeRequestID,
                "params": [
                    "clientInfo": [
                        "name": "codex_quota_widget",
                        "title": "Codex Quota Widget",
                        "version": "0.1.0"
                    ]
                ]
            ])
            scheduleInitializationTimeoutLocked()
        } catch {
            clearProcessLocked()
            emitState(.failed("无法启动 Codex App Server：\(error.localizedDescription)"))
        }
    }

    private func requestRateLimitsLocked() {
        guard didInitialize else { return }
        let requestID = nextRequestID
        nextRequestID += 1
        pendingRateLimitRequestID = requestID

        do {
            try sendLocked([
                "method": "account/rateLimits/read",
                "id": requestID
            ])
            scheduleRateLimitTimeoutLocked(requestID: requestID)
        } catch {
            emitState(.failed("额度查询发送失败：\(error.localizedDescription)"))
        }
    }

    private func sendLocked(_ object: [String: Any]) throws {
        guard let inputHandle else {
            throw ClientError.inputUnavailable
        }
        var data = try JSONSerialization.data(withJSONObject: object)
        data.append(0x0A)
        try inputHandle.write(contentsOf: data)
    }

    private func consumeOutputLocked(_ data: Data) {
        outputBuffer.append(data)

        while let newlineIndex = outputBuffer.firstIndex(of: 0x0A) {
            let line = Data(outputBuffer[..<newlineIndex])
            outputBuffer.removeSubrange(outputBuffer.startIndex...newlineIndex)
            guard !line.isEmpty else { continue }
            handleLineLocked(line)
        }
    }

    private func handleLineLocked(_ data: Data) {
        guard
            let object = try? JSONSerialization.jsonObject(with: data),
            let message = object as? [String: Any]
        else { return }

        if let method = message["method"] as? String {
            if method == "account/rateLimits/updated" {
                // The notification is sparse. Refetch a complete snapshot instead of
                // accidentally clearing fields that are absent from the update.
                ioQueue.asyncAfter(deadline: .now() + 0.4) { [weak self] in
                    self?.requestRateLimitsLocked()
                }
            }
            return
        }

        let responseID = (message["id"] as? NSNumber)?.intValue
        if responseID == initializeRequestID, message["result"] != nil {
            do {
                try sendLocked(["method": "initialized", "params": [:]])
                didInitialize = true
                emitState(.connected)
                requestRateLimitsLocked()
            } catch {
                emitState(.failed("Codex 握手失败：\(error.localizedDescription)"))
            }
            return
        }

        if let errorObject = message["error"] as? [String: Any] {
            let errorMessage = errorObject["message"] as? String ?? "未知 App Server 错误"
            if responseID == pendingRateLimitRequestID {
                pendingRateLimitRequestID = nil
            }
            emitState(.failed(errorMessage))
            return
        }

        guard
            let result = message["result"] as? [String: Any],
            result["rateLimits"] != nil
        else { return }

        do {
            let resultData = try JSONSerialization.data(withJSONObject: result)
            let snapshot = try JSONDecoder().decode(RateLimitsEnvelope.self, from: resultData)
            if responseID == pendingRateLimitRequestID {
                pendingRateLimitRequestID = nil
            }
            emitSnapshot(snapshot)
            emitState(.connected)
        } catch {
            emitState(.failed("无法解析额度数据：\(error.localizedDescription)"))
        }
    }

    private func scheduleInitializationTimeoutLocked() {
        let expectedProcess = process
        ioQueue.asyncAfter(deadline: .now() + 20) { [weak self, weak expectedProcess] in
            guard
                let self,
                let expectedProcess,
                self.process === expectedProcess,
                !self.didInitialize
            else { return }
            self.emitState(.failed("连接 Codex 超时。请确认已经登录 Codex。"))
        }
    }

    private func scheduleRateLimitTimeoutLocked(requestID: Int) {
        ioQueue.asyncAfter(deadline: .now() + 25) { [weak self] in
            guard let self, self.pendingRateLimitRequestID == requestID else { return }
            self.pendingRateLimitRequestID = nil
            self.emitState(.failed("额度查询超时，稍后会自动重试。"))
        }
    }

    private func appendErrorTailLocked(_ text: String) {
        errorTail.append(text)
        if errorTail.count > 2_000 {
            errorTail = String(errorTail.suffix(2_000))
        }
    }

    private func handleTerminationLocked(status: Int32) {
        let intentional = isStopping
        let detail = Self.lastMeaningfulErrorLine(in: errorTail)
        clearProcessLocked()
        guard !intentional else {
            emitState(.disconnected)
            return
        }

        let message = detail.isEmpty
            ? "Codex App Server 已退出（状态码 \(status)）。"
            : "Codex App Server 已退出：\(detail)"
        emitState(.failed(message))
    }

    private func clearProcessLocked() {
        outputPipe?.fileHandleForReading.readabilityHandler = nil
        errorPipe?.fileHandleForReading.readabilityHandler = nil
        process = nil
        inputHandle = nil
        outputPipe = nil
        errorPipe = nil
        didInitialize = false
        pendingRateLimitRequestID = nil
    }

    private func emitSnapshot(_ snapshot: RateLimitsEnvelope) {
        DispatchQueue.main.async { [weak self] in
            self?.onSnapshot?(snapshot)
        }
    }

    private func emitState(_ state: ConnectionState) {
        DispatchQueue.main.async { [weak self] in
            self?.onStateChange?(state)
        }
    }

    private static func locateCodexExecutable() -> URL? {
        let environment = ProcessInfo.processInfo.environment
        var candidates: [String] = []

        if let override = environment["CODEX_BINARY"], !override.isEmpty {
            candidates.append(override)
        }

        candidates.append(contentsOf: [
            "/Applications/ChatGPT.app/Contents/Resources/codex",
            "/Applications/Codex.app/Contents/Resources/codex"
        ])

        if let path = environment["PATH"] {
            candidates.append(contentsOf: path
                .split(separator: ":")
                .map { String($0) + "/codex" })
        }

        return candidates.first(where: FileManager.default.isExecutableFile(atPath:))
            .map(URL.init(fileURLWithPath:))
    }

    private static func lastMeaningfulErrorLine(in text: String) -> String {
        text
            .split(whereSeparator: \Character.isNewline)
            .map(String.init)
            .reversed()
            .first(where: { line in
                !line.contains("WARNING: proceeding") && !line.trimmingCharacters(in: .whitespaces).isEmpty
            }) ?? ""
    }

    private enum ClientError: LocalizedError {
        case inputUnavailable

        var errorDescription: String? {
            switch self {
            case .inputUnavailable:
                return "Codex 输入通道不可用"
            }
        }
    }
}
