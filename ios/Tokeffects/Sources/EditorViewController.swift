import UIKit
import WebKit

final class EditorViewController: UIViewController, WKNavigationDelegate, WKUIDelegate, WKDownloadDelegate {
    private var webView: WKWebView!
    private var server: LocalWebServer?
    private var downloadDestinations: [ObjectIdentifier: URL] = [:]

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black

        let configuration = WKWebViewConfiguration()
        configuration.allowsInlineMediaPlayback = true
        configuration.mediaTypesRequiringUserActionForPlayback = []
        let browser = WKWebView(frame: .zero, configuration: configuration)
        browser.navigationDelegate = self
        browser.uiDelegate = self
        browser.isOpaque = false
        browser.backgroundColor = .black
        browser.scrollView.backgroundColor = .black
        browser.scrollView.isScrollEnabled = false
        browser.scrollView.bounces = false
        browser.scrollView.contentInsetAdjustmentBehavior = .never
        browser.allowsBackForwardNavigationGestures = false
        browser.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(browser)
        NSLayoutConstraint.activate([
            browser.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor),
            browser.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor),
            browser.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            browser.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor)
        ])
        webView = browser

        guard let root = Bundle.main.resourceURL?.appendingPathComponent("web", isDirectory: true) else {
            showStartupError("The bundled Tokeffects web editor is missing.")
            return
        }
        let webServer = LocalWebServer(root: root)
        server = webServer
        webServer.start { [weak self] result in
            DispatchQueue.main.async {
                guard let self else { return }
                switch result {
                case .success(let port):
                    var components = URLComponents()
                    components.scheme = "http"
                    components.host = "127.0.0.1"
                    components.port = Int(port)
                    components.path = "/index.html"
                    components.queryItems = [
                        URLQueryItem(name: "frameworkers", value: "1"),
                        URLQueryItem(name: "nogpuworkers", value: nil),
                        URLQueryItem(name: "nosw", value: nil)
                    ]
                    guard let url = components.url else {
                        self.showStartupError("Could not create the local editor URL.")
                        return
                    }
                    self.webView.load(URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 60))
                case .failure(let error):
                    self.showStartupError("Tokeffects could not start its bundled editor.\n\n\(error.localizedDescription)")
                }
            }
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        UIDevice.current.userInterfaceIdiom == .pad ? .all : [.portrait, .landscapeLeft, .landscapeRight]
    }

    private func isBundledEditorURL(_ url: URL) -> Bool {
        url.scheme == "http" && url.host == "127.0.0.1" && url.port == Int(LocalWebServer.port)
    }

    private func showStartupError(_ message: String) {
        let label = UILabel()
        label.text = message
        label.textColor = .white
        label.font = .systemFont(ofSize: 15, weight: .medium)
        label.textAlignment = .center
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(label)
        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 24),
            label.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -24),
            label.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        ])
    }

    func webView(_ webView: WKWebView,
                 decidePolicyFor navigationAction: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        guard let url = navigationAction.request.url else {
            decisionHandler(.cancel)
            return
        }

        if navigationAction.shouldPerformDownload {
            decisionHandler(.download)
            return
        }
        if isBundledEditorURL(url) || url.scheme == "about" || url.scheme == "blob" {
            decisionHandler(.allow)
            return
        }

        decisionHandler(.cancel)
        if ["http", "https", "mailto"].contains(url.scheme?.lowercased() ?? "") {
            UIApplication.shared.open(url)
        }
    }

    func webView(_ webView: WKWebView,
                 createWebViewWith configuration: WKWebViewConfiguration,
                 for navigationAction: WKNavigationAction,
                 windowFeatures: WKWindowFeatures) -> WKWebView? {
        if let url = navigationAction.request.url, !isBundledEditorURL(url) {
            UIApplication.shared.open(url)
        }
        return nil
    }

    func webView(_ webView: WKWebView, navigationAction: WKNavigationAction, didBecome download: WKDownload) {
        download.delegate = self
    }

    func webView(_ webView: WKWebView, navigationResponse: WKNavigationResponse, didBecome download: WKDownload) {
        download.delegate = self
    }

    func download(_ download: WKDownload,
                  decideDestinationUsing response: URLResponse,
                  suggestedFilename: String,
                  completionHandler: @escaping (URL?) -> Void) {
        let safeName = URL(fileURLWithPath: suggestedFilename).lastPathComponent
        let destination = FileManager.default.temporaryDirectory
            .appendingPathComponent("Tokeffects-Exports", isDirectory: true)
            .appendingPathComponent(safeName)
        do {
            try FileManager.default.createDirectory(
                at: destination.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            try? FileManager.default.removeItem(at: destination)
            downloadDestinations[ObjectIdentifier(download)] = destination
            completionHandler(destination)
        } catch {
            completionHandler(nil)
            showStartupError("Could not prepare the export file: \(error.localizedDescription)")
        }
    }

    func downloadDidFinish(_ download: WKDownload) {
        guard let destination = downloadDestinations.removeValue(forKey: ObjectIdentifier(download)) else { return }
        DispatchQueue.main.async {
            let share = UIActivityViewController(activityItems: [destination], applicationActivities: nil)
            if let popover = share.popoverPresentationController {
                popover.sourceView = self.view
                popover.sourceRect = CGRect(x: self.view.bounds.midX, y: self.view.bounds.midY, width: 1, height: 1)
            }
            self.present(share, animated: true)
        }
    }

    func download(_ download: WKDownload, didFailWithError error: Error, resumeData: Data?) {
        downloadDestinations.removeValue(forKey: ObjectIdentifier(download))
        DispatchQueue.main.async {
            self.showStartupError("The export could not be downloaded: \(error.localizedDescription)")
        }
    }
}
