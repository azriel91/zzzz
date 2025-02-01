#[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4609
#[macro_export]
macro_rules! ws_params_augment {
    ($cmd_ctx_builder:ident) => {
        let $cmd_ctx_builder = $cmd_ctx_builder
            .with_workspace_params_k::<$crate::model::WorkspaceParamsKey>()
            .with_workspace_param::<peace::profile_model::Profile>($crate::model::WorkspaceParamsKey::Profile)
            .with_workspace_param::<$crate::model::EnvManFlow>($crate::model::WorkspaceParamsKey::Flow);
    };
}

#[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4609
#[macro_export]
macro_rules! ws_and_profile_params_augment {
    ($cmd_ctx_builder:ident) => {
        let $cmd_ctx_builder = $cmd_ctx_builder
            .with_workspace_params_k::<$crate::model::WorkspaceParamsKey>()
            .with_workspace_param::<peace::profile_model::Profile>($crate::model::WorkspaceParamsKey::Profile)
            .with_workspace_param::<$crate::model::EnvManFlow>($crate::model::WorkspaceParamsKey::Flow)
            .with_profile_params_k::<$crate::model::ProfileParamsKey>()
            .with_profile_param::<$crate::model::EnvType>($crate::model::ProfileParamsKey::EnvType);
    };
}

#[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4609
#[macro_export]
macro_rules! interruptibility_augment {
    ($cmd_ctx_builder:ident) => {
        let (interrupt_tx, interrupt_rx) = tokio::sync::mpsc::channel::<
            peace::cmd::interruptible::InterruptSignal
        >(16);
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for `SIGINT`.");
            let _ = interrupt_tx.send(peace::cmd::interruptible::InterruptSignal).await;
        });

        let $cmd_ctx_builder = $cmd_ctx_builder
            .with_interruptibility(peace::cmd::interruptible::Interruptibility::new(
                interrupt_rx.into(),
                peace::cmd::interruptible::InterruptStrategy::FinishCurrent,
            ));
    };
}

pub use interruptibility_augment;
pub use ws_and_profile_params_augment;
pub use ws_params_augment;
