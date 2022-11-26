use std::marker::PhantomData;

use peace::cfg::{async_trait, State, StateDiffFnSpec};

use crate::{
    ShCmdData, ShCmdError, ShCmdExecutionRecord, ShCmdExecutor, ShCmdState, ShCmdStateDiff,
};

/// Runs a shell command to obtain the `ShCmd` diff.
#[derive(Debug)]
pub struct ShCmdStateDiffFnSpec<Id>(PhantomData<Id>);

#[async_trait(?Send)]
impl<Id> StateDiffFnSpec for ShCmdStateDiffFnSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = ShCmdData<'op, Id>;
    type Error = ShCmdError;
    type StateDiff = ShCmdStateDiff;
    type StateLogical = ShCmdState;
    type StatePhysical = ShCmdExecutionRecord;

    async fn exec(
        sh_cmd_data: ShCmdData<'_, Id>,
        state_current: &State<ShCmdState, ShCmdExecutionRecord>,
        state_desired: &ShCmdState,
    ) -> Result<Self::StateDiff, ShCmdError> {
        let mut state_desired_sh_cmd = sh_cmd_data.sh_cmd_params().state_desired_sh_cmd().clone();

        let state_current_arg = match &state_current.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let state_desired_arg = match state_desired {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        state_desired_sh_cmd
            .arg(state_current_arg)
            .arg(state_desired_arg);
        ShCmdExecutor::exec(&state_desired_sh_cmd)
            .await
            .map(|state| match state.logical {
                ShCmdState::None => ShCmdStateDiff::new(String::from(""), String::from("")),
                ShCmdState::Some { stdout, stderr } => ShCmdStateDiff::new(stdout, stderr),
            })
    }
}
