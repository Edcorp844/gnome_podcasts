// chapters_page.rs
//
// Copyright 2025-2026 nee <nee-git@patchouli.garden>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use glib::{SignalHandlerId, clone};
use gst::ClockTime;
use gtk::CompositeTemplate;
use gtk::glib;
use mpris_server::PlaybackStatus;
use std::cell::RefCell;

use crate::download_covers::load_widget_texture;
use crate::player::{Duration, Player, PlayerUi, Position};
use crate::utils::format_duration;
use podcasts_data::Episode;
use podcasts_data::ShowCoverModel;

#[derive(Debug, CompositeTemplate, Default)]
#[template(resource = "/org/gnome/Podcasts/gtk/sheet_player.ui")]
pub(crate) struct SheetPlayerPriv {
    #[template_child]
    cover: TemplateChild<gtk::Image>,
    #[template_child]
    play_pause: TemplateChild<gtk::Stack>,
    #[template_child]
    play: TemplateChild<gtk::Button>,
    #[template_child]
    pause: TemplateChild<gtk::Button>,
    #[template_child]
    duration: TemplateChild<gtk::Label>,
    #[template_child]
    progressed: TemplateChild<gtk::Label>,
    #[template_child]
    slider: TemplateChild<gtk::Scale>,
    #[template_child]
    forward: TemplateChild<gtk::Button>,
    #[template_child]
    rewind: TemplateChild<gtk::Button>,
    #[template_child]
    show: TemplateChild<gtk::Label>,
    #[template_child]
    episode: TemplateChild<gtk::Label>,

    // for blocking the signal during duration/position updates
    // as the signal is used to jump when the slider is dragged by a user
    slider_update: RefCell<Option<SignalHandlerId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SheetPlayerPriv {
    const NAME: &'static str = "PdSheetPlayer";
    type Type = SheetPlayer;
    type ParentType = adw::Bin;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl WidgetImpl for SheetPlayerPriv {}
impl ObjectImpl for SheetPlayerPriv {}
impl BinImpl for SheetPlayerPriv {}

glib::wrapper! {
    pub(crate) struct SheetPlayer(ObjectSubclass<SheetPlayerPriv>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SheetPlayer {
    pub fn init(&self, player: &Player) {
        let imp = self.imp();
        imp.slider.set_range(0.0, 1.0);
        let slider_update = Self::connect_update_slider(&imp.slider, player);
        imp.slider_update.replace(Some(slider_update));

        player.bind_ui(self);
    }

    fn connect_update_slider(slider: &gtk::Scale, player: &Player) -> SignalHandlerId {
        slider.connect_value_changed(clone!(
            #[weak]
            player,
            move |slider| {
                let value = slider.value() as u64;
                player.jump_to(Position(ClockTime::from_seconds(value)));
            }
        ))
    }
}

impl PlayerUi for SheetPlayer {
    fn show_cover_changed(&self, show: &ShowCoverModel) {
        load_widget_texture(&self.imp().cover.get(), show.id(), crate::Thumb256, true);
    }

    fn show_cover_reset(&self) {}

    fn show_changed(&self, show: &ShowCoverModel) {
        let imp = self.imp();
        imp.show.set_text(show.title());
        imp.show.set_tooltip_text(Some(show.title()));
    }

    fn episode_changed(&self, ep: &Episode) {
        let imp = self.imp();
        imp.episode.set_text(ep.title());
        imp.episode.set_tooltip_text(Some(ep.title()));
    }

    fn status_changed(&self, status: PlaybackStatus) {
        let stack = &self.imp().play_pause;
        let had_focus = stack
            .visible_child()
            .map(|w| w.is_focus())
            .unwrap_or_default();
        let new_button = match status {
            PlaybackStatus::Paused => self.imp().play.get(),
            PlaybackStatus::Stopped => self.imp().play.get(),
            _ => self.imp().pause.get(),
        };
        stack.set_visible_child(&new_button);
        // restore focus for accessibility
        if had_focus {
            new_button.grab_focus();
        }
    }

    fn position_changed(&self, position: Position) {
        let seconds = position.seconds();
        let imp = self.imp();
        imp.slider
            .block_signal(imp.slider_update.borrow().as_ref().unwrap());
        imp.slider.set_value(seconds as f64);
        imp.slider
            .unblock_signal(imp.slider_update.borrow().as_ref().unwrap());

        imp.progressed.set_text(&format_duration(seconds as u32));
    }

    fn duration_changed(&self, duration: Duration) {
        let seconds = duration.seconds();
        let imp = self.imp();
        imp.slider
            .block_signal(imp.slider_update.borrow().as_ref().unwrap());
        imp.slider.set_range(0.0, seconds as f64);
        imp.slider
            .unblock_signal(imp.slider_update.borrow().as_ref().unwrap());

        imp.duration.set_text(&format_duration(seconds as u32));
    }
}
