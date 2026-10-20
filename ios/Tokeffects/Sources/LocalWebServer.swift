import Foundation
import Network

/// Serves the bundled WASM site on loopback. The stable local origin enables persistent browser
/// storage while keeping the app's HTML/WASM/assets on the device.
final class LocalWebServer {
    static let port: UInt16 = 18473

    private let root: URL
    private let queue = DispatchQueue(label: "com.tokeffects.local-web-server", qos: .userInitiated)
    private var listener: NWListener?
    private var startCompletion: ((Result<UInt16, Error>) -> Void)?
    private var startResolved = false

    init(root: URL) {
        self.root = root.standardizedFileURL
    }

    func start(completion: @escaping (Result<UInt16, Error>) -> Void) {
        startCompletion = completion
        do {
            guard let port = NWEndpoint.Port(rawValue: Self.port) else {
                throw NSError(domain: "Tokeffects.LocalWebServer", code: 1, userInfo: [
                    NSLocalizedDescriptionKey: "The local editor port is invalid."
                ])
            }
            let parameters = NWParameters.tcp
            parameters.requiredLocalEndpoint = .hostPort(
                host: NWEndpoint.Host("127.0.0.1"),
                port: port
            )
            let listener = try NWListener(using: parameters, on: port)
            self.listener = listener
            listener.stateUpdateHandler = { [weak self] state in
                guard let self else { return }
                switch state {
                case .ready:
                    self.finishStart(.success(Self.port))
                case .failed(let error):
                    self.finishStart(.failure(error))
                default:
                    break
                }
            }
            listener.newConnectionHandler = { [weak self] connection in
                guard let self else {
                    connection.cancel()
                    return
                }
                connection.start(queue: self.queue)
                self.receiveHeaders(connection, accumulated: Data())
            }
            listener.start(queue: queue)
        } catch {
            finishStart(.failure(error))
        }
    }

    private func finishStart(_ result: Result<UInt16, Error>) {
        guard !startResolved else { return }
        startResolved = true
        let completion = startCompletion
        startCompletion = nil
        DispatchQueue.main.async {
            completion?(result)
        }
    }

    private func receiveHeaders(_ connection: NWConnection, accumulated: Data) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 8192) { [weak self] data, _, complete, error in
            guard let self else {
                connection.cancel()
                return
            }
            var request = accumulated
            if let data { request.append(data) }
            if request.range(of: Data([13, 10, 13, 10])) != nil {
                self.respond(to: connection, request: request)
            } else if error == nil && !complete && request.count < 32768 {
                self.receiveHeaders(connection, accumulated: request)
            } else {
                connection.cancel()
            }
        }
    }

    private func respond(to connection: NWConnection, request: Data) {
        guard let text = String(data: request, encoding: .utf8),
              let firstLine = text.components(separatedBy: "\r\n").first else {
            sendError(connection, status: "400 Bad Request", message: "Bad request.")
            return
        }
        let parts = firstLine.split(separator: " ")
        guard parts.count >= 2 else {
            sendError(connection, status: "400 Bad Request", message: "Bad request.")
            return
        }
        let method = String(parts[0])
        guard method == "GET" || method == "HEAD" else {
            sendError(connection, status: "405 Method Not Allowed", message: "Only GET and HEAD are supported.")
            return
        }
        let target = String(parts[1])
            .split(separator: "?", maxSplits: 1, omittingEmptySubsequences: false)
            .first
            .map(String.init) ?? "/"
        guard let decoded = target.removingPercentEncoding else {
            sendError(connection, status: "400 Bad Request", message: "Invalid URL path.")
            return
        }
        let components = decoded.split(separator: "/", omittingEmptySubsequences: true).map(String.init)
        guard !components.contains("..") && !components.contains(".") else {
            sendError(connection, status: "403 Forbidden", message: "Forbidden.")
            return
        }

        var fileURL = root
        for component in components {
            fileURL.appendPathComponent(component)
        }
        fileURL = fileURL.standardizedFileURL
        guard fileURL.path.hasPrefix(root.path + "/") || fileURL == root else {
            sendError(connection, status: "403 Forbidden", message: "Forbidden.")
            return
        }
        var isDirectory: ObjCBool = false
        guard FileManager.default.fileExists(atPath: fileURL.path, isDirectory: &isDirectory) else {
            sendError(connection, status: "404 Not Found", message: "Not found.")
            return
        }
        if isDirectory.boolValue {
            fileURL.appendPathComponent("index.html")
        }
        guard FileManager.default.fileExists(atPath: fileURL.path),
              let attributes = try? FileManager.default.attributesOfItem(atPath: fileURL.path),
              let size = (attributes[.size] as? NSNumber)?.int64Value else {
            sendError(connection, status: "404 Not Found", message: "Not found.")
            return
        }

        let headers = [
            "HTTP/1.1 200 OK",
            "Content-Type: \(mimeType(for: fileURL.pathExtension))",
            "Content-Length: \(size)",
            "Connection: close",
            "Cache-Control: no-store",
            "Cross-Origin-Opener-Policy: same-origin",
            "Cross-Origin-Embedder-Policy: require-corp",
            "Cross-Origin-Resource-Policy: same-origin",
            "X-Content-Type-Options: nosniff",
            "\r\n"
        ].joined(separator: "\r\n")

        connection.send(content: Data(headers.utf8), completion: .contentProcessed { [weak self] error in
            guard let self, error == nil else {
                connection.cancel()
                return
            }
            if method == "HEAD" {
                connection.send(
                    content: nil,
                    contentContext: .defaultMessage,
                    isComplete: true,
                    completion: .contentProcessed { _ in connection.cancel() }
                )
                return
            }
            do {
                let handle = try FileHandle(forReadingFrom: fileURL)
                self.sendNextChunk(connection, handle: handle)
            } catch {
                connection.cancel()
            }
        })
    }

    private func sendNextChunk(_ connection: NWConnection, handle: FileHandle) {
        let chunk: Data
        do {
            chunk = try handle.read(upToCount: 64 * 1024) ?? Data()
        } catch {
            try? handle.close()
            connection.cancel()
            return
        }
        guard !chunk.isEmpty else {
            try? handle.close()
            connection.send(
                content: nil,
                contentContext: .defaultMessage,
                isComplete: true,
                completion: .contentProcessed { _ in connection.cancel() }
            )
            return
        }
        connection.send(content: chunk, completion: .contentProcessed { [weak self] error in
            guard let self, error == nil else {
                try? handle.close()
                connection.cancel()
                return
            }
            self.sendNextChunk(connection, handle: handle)
        })
    }

    private func sendError(_ connection: NWConnection, status: String, message: String) {
        let body = Data(message.utf8)
        let header = Data(
            "HTTP/1.1 \(status)\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: \(body.count)\r\nConnection: close\r\n\r\n".utf8
        )
        var response = header
        response.append(body)
        connection.send(content: response, completion: .contentProcessed { _ in connection.cancel() })
    }

    private func mimeType(for extensionName: String) -> String {
        switch extensionName.lowercased() {
        case "html": return "text/html; charset=utf-8"
        case "js", "mjs": return "text/javascript; charset=utf-8"
        case "wasm": return "application/wasm"
        case "css": return "text/css; charset=utf-8"
        case "json": return "application/json"
        case "webmanifest": return "application/manifest+json"
        case "svg": return "image/svg+xml"
        case "png": return "image/png"
        case "jpg", "jpeg": return "image/jpeg"
        case "ico": return "image/x-icon"
        case "woff": return "font/woff"
        case "woff2": return "font/woff2"
        case "txt": return "text/plain; charset=utf-8"
        default: return "application/octet-stream"
        }
    }
}
