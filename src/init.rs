use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::env;
use std::path::PathBuf;

use crate::context;

fn init_log() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {h({l})} | {t} | {m}{n}",
        )))
        .build();

    #[cfg(debug_assertions)]
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();

    #[cfg(not(debug_assertions))]
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();

    log4rs::init_config(config).unwrap();

    Ok(())
}

fn init_context() -> Result<(), Box<dyn std::error::Error>> {
    let template_dirs = vec![
        #[cfg(debug_assertions)]
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates"),
        PathBuf::from(env::var("HOME").unwrap()).join(".local/share/tuack-ng/templates"),
        PathBuf::from("/usr/share/tuack-ng/templates"),
    ];
    context::setup_context(context::Context {
        template_dirs: template_dirs,
    })?;
    Ok(())
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_log()?;
    init_context()?;
    Ok(())
}
