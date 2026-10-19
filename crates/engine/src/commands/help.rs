//! In-app Help menu actions.

use crate::cmd;
use super::{CommandSpec, always};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!("help.inAppTutorials", "In-App Tutorials...", ["Help"], None, "{} → the Home screen's Learn tab", always, |s, p| super::frontend(
            s,
            "help.inAppTutorials",
            p
        )),
    ]
}
