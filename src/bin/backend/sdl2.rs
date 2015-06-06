use super::EmulatorBackend;
use config::EmulatorAppConfig;

pub struct BackendSDL2;

impl EmulatorBackend for BackendSDL2 {
    fn run(&mut self, config: EmulatorAppConfig) {
        info!("starting the UI thread.")
    }
}
