use peace::{
    item_model::item_id,
    progress_model::{
        CmdProgressUpdate, ProgressDelta, ProgressMsgUpdate, ProgressSender, ProgressUpdateAndId,
    },
    rt_model::ProgressUpdate,
};
use tokio::sync::mpsc::{self, error::TryRecvError};

#[test]
fn clone() {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = Clone::clone(&ProgressSender::new(&item_id, &progress_tx));

    progress_sender.inc(123, ProgressMsgUpdate::NoChange);
    progress_rx.close();

    let cmd_progress_update = progress_rx.try_recv().unwrap();

    assert_eq!(
        CmdProgressUpdate::ItemProgress {
            progress_update_and_id: ProgressUpdateAndId {
                item_id: item_id!("test_item_id"),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(123)),
                msg_update: ProgressMsgUpdate::NoChange,
            }
        },
        cmd_progress_update
    );
    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
}

#[test]
fn inc_sends_progress_update() -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_sender.inc(123, ProgressMsgUpdate::NoChange);

    let cmd_progress_update = progress_rx.try_recv().unwrap();

    assert_eq!(
        CmdProgressUpdate::ItemProgress {
            progress_update_and_id: ProgressUpdateAndId {
                item_id: item_id!("test_item_id"),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(123)),
                msg_update: ProgressMsgUpdate::NoChange,
            }
        },
        cmd_progress_update
    );
    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn inc_is_received_if_sent_before_progress_channel_is_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_sender.inc(123, ProgressMsgUpdate::NoChange);
    progress_rx.close();

    let cmd_progress_update = progress_rx.try_recv().unwrap();

    assert_eq!(
        CmdProgressUpdate::ItemProgress {
            progress_update_and_id: ProgressUpdateAndId {
                item_id: item_id!("test_item_id"),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(123)),
                msg_update: ProgressMsgUpdate::NoChange,
            }
        },
        cmd_progress_update
    );
    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn inc_does_not_panic_when_progress_channel_is_closed() -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_rx.close();
    progress_sender.inc(123, ProgressMsgUpdate::NoChange);

    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn tick_sends_progress_update() -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_sender.tick(ProgressMsgUpdate::NoChange);

    let cmd_progress_update = progress_rx.try_recv().unwrap();

    assert_eq!(
        CmdProgressUpdate::ItemProgress {
            progress_update_and_id: ProgressUpdateAndId {
                item_id: item_id!("test_item_id"),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Tick),
                msg_update: ProgressMsgUpdate::NoChange,
            }
        },
        cmd_progress_update
    );
    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn tick_is_received_if_sent_before_progress_channel_is_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_sender.tick(ProgressMsgUpdate::NoChange);
    progress_rx.close();

    let cmd_progress_update = progress_rx.try_recv().unwrap();

    assert_eq!(
        CmdProgressUpdate::ItemProgress {
            progress_update_and_id: ProgressUpdateAndId {
                item_id: item_id!("test_item_id"),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Tick),
                msg_update: ProgressMsgUpdate::NoChange,
            }
        },
        cmd_progress_update
    );
    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn tick_does_not_panic_when_progress_channel_is_closed() -> Result<(), Box<dyn std::error::Error>> {
    let item_id = item_id!("test_item_id");
    let (progress_tx, mut progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    progress_rx.close();
    progress_sender.tick(ProgressMsgUpdate::NoChange);

    let error = progress_rx.try_recv().unwrap_err();
    assert_eq!(TryRecvError::Empty, error);
    Ok(())
}

#[test]
fn debug() {
    let item_id = item_id!("test_item_id");
    let (progress_tx, _progress_rx) = mpsc::channel(10);
    let progress_sender = ProgressSender::new(&item_id, &progress_tx);

    assert!(format!("{progress_sender:?}")
        .starts_with(r#"ProgressSender { item_id: ItemId("test_item_id"), progress_tx: Sender"#));
}
