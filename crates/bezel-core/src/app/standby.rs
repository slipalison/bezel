//! What a screen does when the computer shuts down
//! (D-2026-10-03-power-off-standby-2, -3): showing the choice, changing it
//! (which writes the plan B on the screen and records the choice in the
//! catalog shared by the CLI and the studio) and applying it at shutdown.
//!
//! Rev C screens only; any other family is `Unsupported` before anything is
//! sent. Changing the choice needs the screen and `Confirm::Yes`: with
//! `Confirm::No` nothing is sent, loaded or saved. Applying it takes the
//! choice as the catalog records it ([`recorded_choice`], the only way to a
//! [`RecordedChoice`]) and sends only what it says (TURNOFF, PLAY_VIDEO in
//! a loop, or OPTIONS and RESTART), and TURNOFF instead when the choice
//! cannot be honoured, so that the screen never stays frozen on the last
//! frame.

use crate::app::storage::{Presence, ensure_stored, presence, storage_of};
use crate::domain::archive::ScreenKey;
use crate::domain::media::MediaKind;
use crate::domain::screen::Confirm;
use crate::domain::standby::{
    Offer, PlanB, RecordedChoice, Standby, StandbyOption, Unavailable, supports, unavailable,
};
use crate::domain::storage::{
    BootMedia, Confirmed, Medium, Operation, Refusal, RemotePath, Repeat, StartMode,
    StorageLocation,
};
use crate::ports::{ArchiveStore, ScreenLink, ScreenStorage};
use crate::{BezelError, Result};

/// One screen's choice as [`show`] reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandbyOverview {
    /// The screen's catalog key.
    pub key: ScreenKey,
    /// The recorded choice (`keep` when none was recorded).
    pub standby: Standby,
    /// The plan B the recorded choice writes next to the recorded boot
    /// media ([`crate::domain::archive::ScreenRecord::plan_b`]).
    pub plan_b: PlanB,
    /// What the screen offers: its card and stored videos (nothing for a
    /// family without the choice).
    pub offer: Offer,
    /// The four options and why any cannot be chosen now.
    pub options: [StandbyOption; 4],
}

/// What [`at_shutdown`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// `keep`: nothing was sent.
    Nothing,
    /// `off`: the screen was turned off.
    TurnedOff,
    /// `video`: the stored video loops.
    Video(RemotePath),
    /// `album`: start mode 1 was written and the screen restarts into the
    /// photos of its card.
    Album,
    /// The choice could not be honoured (the video is gone, no card): the
    /// screen was turned off instead of staying frozen.
    TurnedOffInstead(Unavailable),
}

/// `Unsupported` unless the screen behind `link` takes the choice (rev C).
fn ensure_supported(link: &dyn ScreenLink) -> Result<()> {
    let model = link.identity().model;
    if supports(model) {
        return Ok(());
    }
    Err(BezelError::Unsupported(format!(
        "what {} does when the computer shuts down cannot be chosen (rev C screens only)",
        model.name
    )))
}

/// The choice and the boot media recorded for `key` (`keep` and the
/// default start screen without a record). Only reads the store.
fn recorded(store: &mut dyn ArchiveStore, key: &ScreenKey) -> Result<(Standby, BootMedia)> {
    let catalog = store.load()?;
    let record = catalog.screen(key);
    Ok(record.map_or((Standby::Keep, BootMedia::Default), |r| {
        (r.standby.clone(), r.boot_media())
    }))
}

/// The choice of the screen behind `link` (keyed `key` in the catalog of
/// `store`), its plan B, and the four options with why any cannot be chosen
/// now. Rev C: queries only (storage info, then the listing of
/// `internal/video` and, with a card, `sd/video`). Other families: every
/// option `Unsupported`, nothing asked of the screen.
pub fn show(
    link: &mut dyn ScreenLink,
    store: &mut dyn ArchiveStore,
    key: &ScreenKey,
) -> Result<StandbyOverview> {
    let (standby, boot) = recorded(store, key)?;
    let plan_b = standby.plan_b(boot.start_mode());
    let (offer, options) = if supports(link.identity().model) {
        let offer = offer(storage_of(link)?)?;
        let options = offer.options();
        (offer, options)
    } else {
        (Offer::default(), unavailable(Unavailable::Unsupported))
    };
    Ok(StandbyOverview {
        key: key.clone(),
        standby,
        plan_b,
        offer,
        options,
    })
}

/// The card and the stored videos (internal first), by queries only; a
/// card's folder is listed only with a card in (listing creates it).
fn offer(storage: &mut dyn ScreenStorage) -> Result<Offer> {
    let info = storage.info()?;
    let mut videos = Vec::new();
    for medium in Medium::ALL {
        if info.capacity(medium).is_none() {
            continue;
        }
        let location = StorageLocation::new(medium, MediaKind::Video);
        let names = storage.list(location)?;
        videos.extend(names.into_iter().map(|n| RemotePath::new(location, n)));
    }
    Ok(Offer {
        card: info.card.is_some(),
        videos,
    })
}

/// Changes the choice of the screen behind `link` to `standby`: writes its
/// plan B on the screen (OPTIONS whole, [`Standby::plan_b`] next to the
/// recorded boot media), then records the choice under `key` in the
/// catalog of `store`, and returns the plan B written
/// (D-2026-10-03-power-off-standby-2 (3)).
///
/// `Confirm::No`: nothing sent, loaded or saved. A family without the
/// choice: `Unsupported`, nothing sent. From `keep` to `keep`: nothing sent
/// nor saved. Before writing, the screen is asked whether the choice can be
/// honoured: `video` needs its file stored (`InvalidInput` otherwise),
/// `album` a card (`Refused(NoCard)`). The catalog is read again right
/// before it is saved, so another writer's changes stay.
pub fn choose(
    link: &mut dyn ScreenLink,
    store: &mut dyn ArchiveStore,
    key: &ScreenKey,
    standby: Standby,
    confirm: Confirm,
) -> Result<PlanB> {
    let confirmed = Confirmed::require(confirm, &Operation::Standby(standby.clone()))?;
    ensure_supported(link)?;
    let (current, boot) = recorded(store, key)?;
    let plan = standby.plan_b(boot.start_mode());
    if current == Standby::Keep && standby == Standby::Keep {
        return Ok(plan);
    }
    let storage = storage_of(link)?;
    honoured(storage, &standby)?;
    storage.set_options(plan, confirmed)?;
    let mut catalog = store.load()?;
    catalog.screen_mut(key).standby = standby;
    store.save(&catalog)?;
    Ok(plan)
}

/// Whether the screen can honour `standby` now (queries only): `video`
/// needs its file stored in a video folder, `album` a card.
fn honoured(storage: &mut dyn ScreenStorage, standby: &Standby) -> Result<()> {
    match standby {
        Standby::Keep | Standby::Off(_) => Ok(()),
        Standby::Video(path) => {
            if path.location.kind != MediaKind::Video {
                return Err(BezelError::InvalidInput(format!("{path} is not a video")));
            }
            ensure_stored(storage, path)
        }
        Standby::Album => match storage.info()?.card {
            Some(_) => Ok(()),
            None => Err(BezelError::Refused(Refusal::NoCard)),
        },
    }
}

/// The choice the catalog of `store` records for `key` (`keep` without a
/// record), as [`at_shutdown`] applies it: the only way to a
/// [`RecordedChoice`]. Only reads the store.
pub fn recorded_choice(store: &mut dyn ArchiveStore, key: &ScreenKey) -> Result<RecordedChoice> {
    let catalog = store.load()?;
    Ok(RecordedChoice::of(catalog.screen(key)))
}

/// Applies `choice`, the choice recorded for the screen behind `link`
/// ([`recorded_choice`]), as the computer shuts down
/// (D-2026-10-03-power-off-standby-3), sending only what the choice says
/// and waiting for nothing the screen does afterwards:
/// - `keep`: nothing;
/// - `off`: [`ScreenLink::turn_off_now`];
/// - `video`: a size query, then its file loops ([`ScreenStorage::play_video`]
///   with [`Repeat::Loop`]);
/// - `album`: a storage info query, then the plan B of `album` (start mode
///   1, no timer, [`ScreenStorage::set_options`]) and
///   [`ScreenStorage::restart`];
/// - a video that is gone or an album without a card: `turn_off_now`
///   instead ([`Applied::TurnedOffInstead`]).
///
/// The OPTIONS and RESTART run under the confirmation the user gave when
/// the choice was recorded (D-2026-10-03-power-off-standby-2 (5)), which
/// only a [`RecordedChoice`] carries. A family without the choice:
/// `Unsupported`, nothing sent.
pub fn at_shutdown(link: &mut dyn ScreenLink, choice: &RecordedChoice) -> Result<Applied> {
    let standby = choice.standby();
    if *standby == Standby::Keep {
        return Ok(Applied::Nothing);
    }
    ensure_supported(link)?;
    match standby {
        Standby::Keep => Ok(Applied::Nothing),
        Standby::Off(_) => {
            link.turn_off_now()?;
            Ok(Applied::TurnedOff)
        }
        Standby::Video(path) => loop_video(link, path),
        Standby::Album => restart_into_album(link, choice),
    }
}

/// `video` at shutdown: loops `path` when it is stored.
fn loop_video(link: &mut dyn ScreenLink, path: &RemotePath) -> Result<Applied> {
    let storage = storage_of(link)?;
    let stored =
        path.location.kind == MediaKind::Video && presence(storage, path)? != Presence::Absent;
    if !stored {
        return turned_off_instead(link, Unavailable::NoVideo);
    }
    storage.play_video(path, Repeat::Loop)?;
    Ok(Applied::Video(path.clone()))
}

/// `album` at shutdown: start mode 1 and a restart, with a card.
fn restart_into_album(link: &mut dyn ScreenLink, choice: &RecordedChoice) -> Result<Applied> {
    let storage = storage_of(link)?;
    if storage.info()?.card.is_none() {
        return turned_off_instead(link, Unavailable::NoCard);
    }
    let plan = choice.standby().plan_b(StartMode::Default);
    storage.set_options(plan, Confirmed::recorded(choice))?;
    storage.restart(Confirmed::recorded(choice))?;
    Ok(Applied::Album)
}

/// A choice that cannot be honoured: the screen is turned off instead.
fn turned_off_instead(link: &mut dyn ScreenLink, reason: Unavailable) -> Result<Applied> {
    link.turn_off_now()?;
    Ok(Applied::TurnedOffInstead(reason))
}
