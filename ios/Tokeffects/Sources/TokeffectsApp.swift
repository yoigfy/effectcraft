import SwiftUI

@main
struct TokeffectsApp: App {
    var body: some Scene {
        WindowGroup {
            EditorHost()
                .preferredColorScheme(.dark)
        }
    }
}

private struct EditorHost: UIViewControllerRepresentable {
    func makeUIViewController(context: Context) -> EditorViewController {
        EditorViewController()
    }

    func updateUIViewController(_ uiViewController: EditorViewController, context: Context) {}
}
