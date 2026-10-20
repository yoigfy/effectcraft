# Bundled web editor

The iOS build workflow generates the optimized web editor, then copies its files to this directory
before invoking XcodeGen. This keeps the web app bundled inside the IPA and aligned with the same
Git commit as the native shell.
