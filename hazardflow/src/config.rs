//! Config

// impl Options {
//     /// Set the global config
//     pub fn initialize() {
//         let mut config = Options::parse();
//
//         if config.display && (config.wire_cache || config.deadcode || config.inline_always) {
//             log!(INFO, "Warning: Display mode is enabled, disabling optimizations");
//             config.wire_cache = false;
//             config.deadcode = false;
//             config.inline_always = false;
//         }
//
//         INSTANCE.set(config).unwrap()
//     }
//
//     pub(crate) fn global() -> &'static Options { INSTANCE.get().unwrap() }
// }
