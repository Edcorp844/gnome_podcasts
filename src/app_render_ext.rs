use adw::prelude::*;
use relm4::ComponentSender;

use crate::{
    app::{AppModel, AppModelInput, AppModelWidgets},
    app_navigation_ext::NavigationPage,
};

impl AppModel {
    pub(crate) fn render_sidebar_list(widgets: &AppModelWidgets, sender: &ComponentSender<Self>) {
        // Data configurations
        let pages_list_items = [
            ("edit-find-symbolic", "Search"),
            ("user-home-symbolic", "Home"),
            ("view-grid-symbolic", "New"),
        ];

        let library_list_items = [
            ("emoji-recent-symbolic", "Recently updated"),
            ("display-projector-symbolic", "Shows"),
            ("folder-download-symbolic", "Downloaded"),
            ("preferences-system-time-symbolic", "History"),
            ("view-list-symbolic", "Channels"),
        ];

        // 1. Clean dynamic population using a unified helper function
        for (icon, label) in &pages_list_items {
            let row = Self::create_sidebar_row(icon, label);
            widgets.pages.append(&row);
        }

        for (icon, label) in &library_list_items {
            let row = Self::create_sidebar_row(icon, label);
            widgets.library.append(&row);
        }

        Self::setup_collapsible_section(
            &widgets.library_header,
            &widgets.library_revealer,
            &widgets.library_chevron,
        );

        let pages_weak = widgets.pages.downgrade();
        let sender_clone = sender.clone();
        widgets.library.connect_row_activated(move |_, row| {
            if let Some(pages) = pages_weak.upgrade() {
                pages.unselect_all();
            }

            let widget_name = row.widget_name().to_string();
            let resolved_page = NavigationPage::from_name(&widget_name);
            sender_clone.input(AppModelInput::SelectPage(resolved_page));
        });

        let library_weak = widgets.library.downgrade();
        let sender_clone = sender.clone();
        widgets.pages.connect_row_activated(move |_, row| {
            if let Some(library) = library_weak.upgrade() {
                library.unselect_all();
            }
            let widget_name = row.widget_name().to_string();
            let resolved_page = NavigationPage::from_name(&widget_name);
            sender_clone.input(AppModelInput::SelectPage(resolved_page));
        });
    }

    fn create_sidebar_row(icon_name: &str, label_text: &str) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();

        let layout_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        layout_box.set_margin_start(8);
        layout_box.set_margin_end(8);
        layout_box.set_margin_top(8);
        layout_box.set_margin_bottom(8);

        let icon = gtk::Image::from_icon_name(icon_name);
        layout_box.append(&icon);

        let label = gtk::Label::new(Some(label_text));
        layout_box.append(&label);

        row.set_child(Some(&layout_box));
        row.set_widget_name(label_text);
        row
    }

    pub fn setup_collapsible_section(
        header: &gtk::Box,
        revealer: &gtk::Revealer,
        chevron: &gtk::Image,
    ) {
        let r = revealer.clone();
        let c = chevron.clone();
        let gesture = gtk::GestureClick::new();

        gesture.connect_released(move |_, _, _, _| {
            let is_revealing = !r.reveals_child();
            r.set_reveal_child(is_revealing);
            c.set_icon_name(Some(if is_revealing {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            }));
        });
        header.add_controller(gesture);
    }
}
