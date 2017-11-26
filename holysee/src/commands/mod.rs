pub mod command_dispatcher;
pub mod karma;
pub mod relay;
pub mod last_seen;
pub mod quote;
pub mod url_preview;
pub mod usage;


// DISABLED DUE TO REFACTOR
//#[cfg(test)]
//mod tests {
//    use commands::command_dispatcher::CommandDispatcher;
//    use settings::Settings;
//
//    #[test]
//    fn is_command_enabled_ok() {
//        let settings = match Settings::new(false) {
//            Ok(s) => s,
//            Err(e) => {
//                panic!("Error accessing config file: {}", e);
//            }
//        };
//        let command_dispatcher = CommandDispatcher::new(&settings.commands);
//        assert!(command_dispatcher.is_command_enabled("relay"));
//    }
//
//    #[test]
//    #[should_panic]
//    fn is_command_enabled_fail() {
//        let settings = match Settings::new(false) {
//            Ok(s) => s,
//            Err(e) => {
//                panic!("Error accessing config file: {}", e);
//            }
//        };
//        let command_dispatcher = CommandDispatcher::new(&settings.commands);
//        assert!(command_dispatcher.is_command_enabled("karma"));
//    }
//}
