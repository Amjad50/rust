use crate::spec::{Cc, LinkerFlavor, Lld, PanicStrategy, TargetOptions};

pub fn opts() -> TargetOptions {
    TargetOptions {
        os: "emerald".into(),
        linker: Some("rust-lld".into()),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        // tls_model: TlsModel::InitialExec,
        // position_independent_executables: false,
        // static_position_independent_executables: false,
        // has_thread_local: false,
        panic_strategy: PanicStrategy::Unwind,
        ..Default::default()
    }
}
