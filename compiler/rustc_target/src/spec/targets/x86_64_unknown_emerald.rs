use crate::spec::{base, StackProbeType, Target};

pub fn target() -> Target {
    let mut base = base::emerald::opts();
    base.cpu = "x86-64".into();
    base.plt_by_default = false;
    base.max_atomic_width = Some(64);
    base.stack_probes = StackProbeType::Inline;

    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
        arch: "x86_64".into(),
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
            .into(),
        metadata: crate::spec::TargetMetadata {
            description: None,
            tier: None,
            host_tools: None,
            std: None,
        },
        options: base,
    }
}
