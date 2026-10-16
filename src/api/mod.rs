pub mod client;
pub mod control;
mod event_hub;
pub mod schema;
mod server;
mod status;
mod subscriptions;
mod wait;

pub use event_hub::EventHub;
#[cfg(test)]
pub(crate) use server::control_stream::terminal_input as control_stream_terminal_input;
pub(crate) use server::default_capabilities as default_server_capabilities;
pub use server::ServerHandle;
pub(crate) use server::{api_method_name, start_server_with_stop_control};
pub use status::{read_runtime_status_at, RuntimeStatus};

use std::path::PathBuf;

use tokio::sync::mpsc;

use crate::api::schema::Request;

pub const SOCKET_PATH_ENV_VAR: &str = "HERDR_SOCKET_PATH";

pub(crate) fn request_changes_ui(request: &Request) -> bool {
    request.method.operation().changes_ui
}

pub struct ApiRequestMessage {
    pub request: Request,
    pub respond_to: std::sync::mpsc::Sender<String>,
    pub response_write_complete: Option<std::sync::mpsc::Receiver<()>>,
    /// Set when the request arrived on a control stream.
    pub control: Option<control::ControlConnectionHandle>,
}

pub type ApiRequestSender = mpsc::UnboundedSender<ApiRequestMessage>;

pub fn socket_path() -> PathBuf {
    crate::session::active_api_socket_path()
}
