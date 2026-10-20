# Tokeffects for iPhone and iPad

Tokeffects for iOS packages the existing WebAssembly editor inside a native SwiftUI / WKWebView
shell. The editor files are bundled in the app; a loopback-only HTTP server provides a stable local
origin for WebAssembly, workers and persistent browser storage. Exported files are handed to the
iOS share sheet.

## Build

The **Build Tokeffects for iOS** GitHub Actions workflow builds the web editor, generates an Xcode
project, compiles the native wrapper and uploads an IPA artifact.

The default workflow output is an **unsigned IPA**. iOS will not install an unsigned IPA on a normal
device. To request an ad-hoc signed IPA, configure these GitHub repository secrets:

- `IOS_CERTIFICATE_P12_BASE64`: base64 of a compatible Apple development/distribution .p12
- `IOS_CERTIFICATE_PASSWORD`: password for that .p12
- `IOS_PROVISIONING_PROFILE_BASE64`: base64 of an ad-hoc provisioning profile for bundle ID `com.tokeffects.editor`
- `IOS_TEAM_ID`: Apple Developer Team ID

The provisioning profile must include the iPhone/iPad devices used to install the app. Never commit
certificates, passwords or provisioning profiles to this repository.
