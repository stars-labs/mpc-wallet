//! Integration tests for `elm::update::update`.
//!
//! These exercise the pure `(Model, Message) → (Model, Option<Command>)`
//! function — no network, no TTY, no async — so they run in milliseconds
//! and can be executed by `cargo test -p tui-node --test update_transitions`.
//!
//! Scope: the DKG-phase state transitions added in commit 442549d. The smoke
//! test (`scripts/smoke-dkg.sh`) covers the same behaviour end-to-end in ~30s;
//! these run in under a second and catch regressions before the smoke test
//! is even worth booting.

use tui_node::elm::command::Command;
use tui_node::elm::message::{DKGRound, Message};
use tui_node::elm::update::update;
use tui_node::elm::{Model, Screen};

fn fresh_model() -> Model {
    Model::new("test-device".to_string())
}

/// A `Command::Batch` returned from `update` should contain `StartFrostProtocol`
/// somewhere among its children. We don't care about order / trailing
/// `ForceRemount` / future additions — just that the FROST trigger is present.
fn batch_contains_start_frost(cmd: &Option<Command>) -> bool {
    let Some(cmd) = cmd else { return false };
    match cmd {
        Command::StartFrostProtocol => true,
        Command::Batch(children) => children
            .iter()
            .any(|c| matches!(c, Command::StartFrostProtocol)),
        _ => false,
    }
}

// -----------------------------------------------------------------
// StartDKGProtocol — mesh is ready, we must enter Round 1
// -----------------------------------------------------------------
#[test]
fn start_dkg_protocol_enters_round1_and_flips_in_progress() {
    let mut model = fresh_model();
    assert_eq!(
        model.wallet_state.dkg_round,
        DKGRound::Initialization,
        "new model should start at Initialization"
    );
    assert!(
        !model.wallet_state.dkg_in_progress,
        "new model should not be running DKG"
    );

    let cmd = update(&mut model, Message::StartDKGProtocol);

    assert_eq!(
        model.wallet_state.dkg_round,
        DKGRound::Round1,
        "StartDKGProtocol should advance dkg_round to Round1"
    );
    assert!(
        model.wallet_state.dkg_in_progress,
        "StartDKGProtocol should set dkg_in_progress so subsequent triggers dedupe"
    );
    assert!(
        batch_contains_start_frost(&cmd),
        "StartDKGProtocol must dispatch Command::StartFrostProtocol (bare or inside Batch) — \
         this was the regression that made the UI sit on 'Initialization' forever"
    );
}

// -----------------------------------------------------------------
// ProcessDKGRound2 — first one flips us out of Round1 into Round2
// -----------------------------------------------------------------
#[test]
fn first_process_dkg_round2_advances_to_round2() {
    let mut model = fresh_model();
    // Simulate having passed through the Round 1 broadcast.
    model.wallet_state.dkg_round = DKGRound::Round1;
    model.wallet_state.dkg_in_progress = true;

    let _ = update(
        &mut model,
        Message::ProcessDKGRound2 {
            from_device: "peer-alice".to_string(),
            package_bytes: vec![0u8; 32], // content doesn't matter; update() doesn't deserialize
        },
    );

    assert_eq!(
        model.wallet_state.dkg_round,
        DKGRound::Round2,
        "first ProcessDKGRound2 while in Round1 should advance the UI label"
    );
}

#[test]
fn subsequent_process_dkg_round2_does_not_regress_round() {
    let mut model = fresh_model();
    model.wallet_state.dkg_round = DKGRound::Round2;

    let _ = update(
        &mut model,
        Message::ProcessDKGRound2 {
            from_device: "peer-bob".to_string(),
            package_bytes: vec![],
        },
    );

    assert_eq!(
        model.wallet_state.dkg_round,
        DKGRound::Round2,
        "ProcessDKGRound2 while already in Round2 must NOT regress or loop"
    );
}

#[test]
fn process_dkg_round2_does_not_override_finalization_or_complete() {
    // Unlikely timing (R2 packet arrives after part3 already ran locally) but
    // the guard must hold anyway — we should not rewind Complete back to Round2.
    for terminal in [DKGRound::Finalization, DKGRound::Complete] {
        let mut model = fresh_model();
        model.wallet_state.dkg_round = terminal.clone();
        let _ = update(
            &mut model,
            Message::ProcessDKGRound2 {
                from_device: "peer-late".to_string(),
                package_bytes: vec![],
            },
        );
        assert_eq!(
            model.wallet_state.dkg_round, terminal,
            "late ProcessDKGRound2 must not rewind {:?}",
            terminal
        );
    }
}

// -----------------------------------------------------------------
// DKGKeyGenerated — terminal state: Complete + clear in_progress + notify
// -----------------------------------------------------------------
#[test]
fn dkg_key_generated_transitions_to_complete() {
    let mut model = fresh_model();
    model.wallet_state.dkg_round = DKGRound::Round2;
    model.wallet_state.dkg_in_progress = true;

    let sample_hex =
        "021de2d69979f0a03ea413e7ed6a32ad02111b90d1f03793649157d3e4ee952143".to_string();
    let before_notif_count = model.ui_state.notifications.len();

    let _ = update(
        &mut model,
        Message::DKGKeyGenerated {
            group_pubkey_hex: sample_hex.clone(),
        },
    );

    assert_eq!(
        model.wallet_state.dkg_round,
        DKGRound::Complete,
        "DKGKeyGenerated must land the UI on 100% Complete, not 95% Finalization \
         — the stuck-at-95% regression"
    );
    assert!(
        !model.wallet_state.dkg_in_progress,
        "DKGKeyGenerated must clear dkg_in_progress so a subsequent wallet flow can start"
    );

    assert_eq!(
        model.ui_state.notifications.len(),
        before_notif_count + 1,
        "DKGKeyGenerated should push exactly one success notification"
    );
    let notif = model.ui_state.notifications.last().unwrap();
    assert!(
        notif.text.contains(&sample_hex[..16]),
        "notification text should embed the 16-char group-key prefix; got {:?}",
        notif.text
    );
}

#[test]
fn dkg_key_generated_on_progress_screen_emits_force_remount() {
    // On the DKG Progress screen the component is rebuilt from Model on every
    // remount, so we need a ForceRemount to paint the new "Complete" state.
    let mut model = fresh_model();
    model.current_screen = Screen::DKGProgress {
        session_id: "dkg-test".to_string(),
    };
    let cmd = update(
        &mut model,
        Message::DKGKeyGenerated {
            group_pubkey_hex: "00".repeat(33),
        },
    );
    match cmd {
        Some(Command::SendMessage(Message::ForceRemount)) => {}
        other => panic!(
            "expected Some(Command::SendMessage(ForceRemount)) on DKGProgress, got {:?}",
            other
        ),
    }
}

#[test]
fn dkg_key_generated_off_progress_screen_does_not_force_remount() {
    // If the user already navigated away (shouldn't happen normally, but
    // guard anyway) we shouldn't send a stray remount.
    let mut model = fresh_model();
    model.current_screen = Screen::MainMenu;
    let cmd = update(
        &mut model,
        Message::DKGKeyGenerated {
            group_pubkey_hex: "00".repeat(33),
        },
    );
    assert!(
        cmd.is_none(),
        "DKGKeyGenerated off the progress screen should return None; got {:?}",
        cmd
    );
}
