use super::App;
use crate::api::schema::ForegroundProcessInfo;
use crate::layout::PaneId;

impl App {
    pub(crate) fn sync_foreground_process(
        &mut self,
        pane_id: PaneId,
        process: Option<ForegroundProcessInfo>,
    ) -> bool {
        let Some((ws_idx, terminal_id)) = self
            .find_pane(pane_id)
            .map(|(ws_idx, pane)| (ws_idx, pane.attached_terminal_id.clone()))
        else {
            return false;
        };
        let Some(terminal) = self.state.terminals.get_mut(&terminal_id) else {
            return false;
        };
        if !terminal.set_foreground_process(process) {
            return false;
        }
        self.emit_pane_updated(ws_idx, pane_id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::events::AppEvent;
    use crate::workspace::Workspace;

    fn pane_updated_events(event_hub: &crate::api::EventHub) -> usize {
        event_hub
            .events_after(0)
            .iter()
            .filter(|(_, event)| event.event == crate::api::schema::EventKind::PaneUpdated)
            .count()
    }

    fn process(name: &str, argv: &[&str], is_shell: bool) -> ForegroundProcessInfo {
        ForegroundProcessInfo {
            name: name.to_string(),
            argv: argv.iter().map(|arg| arg.to_string()).collect(),
            is_shell,
        }
    }

    #[tokio::test]
    async fn foreground_process_changes_emit_one_pane_update_each() {
        let event_hub = crate::api::EventHub::default();
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(
            &Config::default(),
            crate::app::AppPolicy::TEST,
            None,
            api_rx,
            event_hub.clone(),
        );
        app.state.workspaces = vec![Workspace::test_new("one")];
        app.state.active = Some(0);
        app.state.ensure_test_terminals();
        let pane_id = app.state.workspaces[0].tabs[0].root_pane;
        let revision = app.pane_info(0, pane_id).unwrap().revision;

        let shell = process("zsh", &["-zsh"], true);
        app.handle_internal_event(AppEvent::ForegroundProcessChanged {
            pane_id,
            process: Some(shell.clone()),
        });
        let pane = app.pane_info(0, pane_id).unwrap();
        assert_eq!(pane.foreground_process.as_ref(), Some(&shell));
        assert_eq!(pane.revision, revision + 1);
        assert_eq!(pane_updated_events(&event_hub), 1);

        assert!(!app.sync_foreground_process(pane_id, Some(shell.clone())));
        assert_eq!(app.pane_info(0, pane_id).unwrap().revision, revision + 1);
        assert_eq!(pane_updated_events(&event_hub), 1);

        let htop = process("htop", &["htop", "-d", "5"], false);
        assert!(app.sync_foreground_process(pane_id, Some(htop.clone())));
        assert_eq!(
            app.pane_info(0, pane_id).unwrap().foreground_process,
            Some(htop.clone())
        );
        assert_eq!(pane_updated_events(&event_hub), 2);

        let htop_other_args = process("htop", &["htop"], false);
        assert!(app.sync_foreground_process(pane_id, Some(htop_other_args)));
        assert_eq!(pane_updated_events(&event_hub), 3);

        assert!(app.sync_foreground_process(pane_id, None));
        let pane = app.pane_info(0, pane_id).unwrap();
        assert_eq!(pane.foreground_process, None);
        assert_eq!(pane.revision, revision + 4);
        assert_eq!(pane_updated_events(&event_hub), 4);
    }

    #[test]
    fn foreground_process_serializes_compactly() {
        let value = serde_json::to_value(process("nvim", &[], false)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"name": "nvim", "is_shell": false})
        );
    }
}
