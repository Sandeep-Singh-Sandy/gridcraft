use super::controls::{act, anchor_keytip, checkbox_on, control_keytip, invoke, keytip, menu_items, sep};
use super::galleries::{margins_gallery, size_gallery};
use super::{Color32, Icon, Sense, SheetApp, Tokens, Ui, big_button, json, keytip_table, vec2};

pub(super) fn page_layout(app: &mut SheetApp, ui: &mut Ui) {
    let th = big_button(ui, Icon::Theme, "Themes", "Themes", true);
    anchor_keytip(app, ui, &th, &["pageLayout.theme"]);
    egui::Popup::menu(&th).show(|ui| {
        for (name, colors) in gridcraft_engine::cmd::view::themes() {
            ui.horizontal(|ui| {
                for c in colors.iter().skip(4).take(6) {
                    let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                    ui.painter().rect_filled(r, 1.0, Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, *c as u8));
                }
                if ui.button(name).clicked() {
                    act(app, "pageLayout.theme", json!({"name": name}));
                }
            });
        }
    });
    sep(ui);
    let m = big_button(ui, Icon::Margins, "Margins", "Margins", true);
    for sequence in keytip_table::group_extension(app.keytips.prefix().unwrap_or_default(), &["pageLayout.margins", "dialog:pageSetup"]) {
        keytip(app, ui, sequence, m.rect.center_bottom());
    }
    egui::Popup::menu(&m).show(|ui| margins_gallery(app, ui));
    let o = big_button(ui, Icon::Orient, "Page Layout|Orientation", "Page Layout|Orientation", true);
    control_keytip(app, ui, &o, "pageLayout.orientation");
    egui::Popup::menu(&o).show(|ui| {
        menu_items(
            app,
            ui,
            &[
                ("Portrait", "pageLayout.orientation", json!({"orientation": "portrait"})),
                ("Landscape", "pageLayout.orientation", json!({"orientation": "landscape"})),
            ],
        );
    });
    let s = big_button(ui, Icon::Paper, "Size", "Paper Size", true);
    control_keytip(app, ui, &s, "pageLayout.size");
    egui::Popup::menu(&s).show(|ui| size_gallery(app, ui));
    let pa = big_button(ui, Icon::PrintArea, "Print\nArea", "Print Area", true);
    anchor_keytip(app, ui, &pa, &["pageLayout.printArea"]);
    egui::Popup::menu(&pa).show(|ui| {
        menu_items(
            app,
            ui,
            &[("Set Print Area", "pageLayout.printArea", json!({})), ("Clear Print Area", "pageLayout.printArea", json!({"clear": true}))],
        );
    });
    let br = big_button(ui, Icon::PageBreak, "Breaks", "Page Breaks", true);
    for sequence in keytip_table::group_extension(app.keytips.prefix().unwrap_or_default(), &["pageLayout.breaks"]) {
        keytip(app, ui, sequence, br.rect.center_bottom());
    }
    egui::Popup::menu(&br).show(|ui| {
        menu_items(
            app,
            ui,
            &[
                ("Insert Page Break", "pageLayout.breaks", json!({"insert": true})),
                ("Remove Page Break", "pageLayout.breaks", json!({"remove": true})),
                ("Reset All Page Breaks", "pageLayout.breaks", json!({"reset": true})),
            ],
        );
    });
    let __k = big_button(ui, Icon::Sheet, "Print\nTitles", "Print Titles", false);
    control_keytip(app, ui, &__k, "pageLayout.printTitles");
    if __k.clicked() {
        app.open_dialog("pageSetup", json!({"tab": "Sheet"}));
    }
    sep(ui);
    scale_to_fit(app, ui);
    sep(ui);
    let sh = app
        .session
        .active()
        .and_then(|d| d.wb.active().map(|s| (s.show_gridlines, s.show_headings, s.print.gridlines, s.print.headings)))
        .unwrap_or((true, true, false, false));
    // These two checkboxes mean the screen view vs. the printed page. English shows "View" (as in
    // Excel), but the key is distinct from the "View" tab so languages can word them apart.
    let screen = app.ui.language.tr("Sheet Options|View");
    let print = app.ui.language.tr("Print");
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(app.ui.language.tr("Gridlines")).strong().small());
        let mut v = sh.0;
        let resp = ui.checkbox(&mut v, screen);
        control_keytip(app, ui, &resp, "view.gridlines");
        if resp.changed() || invoke(app, "view.gridlines") {
            act(app, "view.gridlines", json!({"on": checkbox_on(resp.changed(), v)}));
        }
        let mut p = sh.2;
        let resp = ui.checkbox(&mut p, print);
        control_keytip(app, ui, &resp, "pageLayout.printGridlines");
        if resp.changed() || invoke(app, "pageLayout.printGridlines") {
            act(app, "pageLayout.printGridlines", json!({"on": checkbox_on(resp.changed(), p)}));
        }
    });
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(app.ui.language.tr("Headings")).strong().small());
        let mut v = sh.1;
        let resp = ui.checkbox(&mut v, screen);
        control_keytip(app, ui, &resp, "view.headings");
        if resp.changed() || invoke(app, "view.headings") {
            act(app, "view.headings", json!({"on": checkbox_on(resp.changed(), v)}));
        }
        let mut p = sh.3;
        let resp = ui.checkbox(&mut p, print);
        control_keytip(app, ui, &resp, "pageLayout.printHeadings");
        if resp.changed() || invoke(app, "pageLayout.printHeadings") {
            act(app, "pageLayout.printHeadings", json!({"on": checkbox_on(resp.changed(), p)}));
        }
    });
    let rtl = app.session.active().and_then(|d| d.wb.active()).is_some_and(|s| s.right_to_left);
    let r = big_button(ui, Icon::SheetRtl, "Sheet\nRight-to-Left", "Sheet Right-to-Left", false);
    if rtl {
        let t = Tokens::get(ui.ctx());
        ui.painter().rect_stroke(r.rect, 5.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    }
    if r.clicked() {
        act(app, "view.rightToLeft", json!({"on": !rtl}));
    }
}

/// The Scale to Fit group: Width and Height in pages (Automatic, 1–9, or more in Page Setup) and
/// a Scale percentage that is only available while both are Automatic, as in Excel.
fn scale_to_fit(app: &mut SheetApp, ui: &mut Ui) {
    let lang = app.ui.language;
    let t = Tokens::get(ui.ctx());
    let (fit, scale) = app.session.active().and_then(|d| d.wb.active()).map_or((None, 100), |s| (s.print.fit_to, s.print.scale));
    let pages = |n: u16| match n {
        0 => lang.tr("Automatic").to_string(),
        1 => lang.tr("1 page").to_string(),
        n => format!("{n} {}", lang.tr("pages")),
    };
    ui.vertical(|ui| {
        for (key, label, now) in [("width", "Width:", fit.map_or(0, |f| f.0)), ("height", "Height:", fit.map_or(0, |f| f.1))] {
            ui.horizontal(|ui| {
                let name = ui.add_sized([44.0, 20.0], egui::Label::new(lang.tr(label)));
                craft_ui::ComboBox::from_id_salt(("scale_to_fit", key))
                    .selected_text(pages(now))
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        for n in 0..=9 {
                            if ui.selectable_label(now == n, pages(n)).clicked() {
                                act(app, "pageLayout.scaleToFit", json!({ (key): n }));
                            }
                        }
                        if ui.selectable_label(false, lang.tr("More Pages…")).clicked() {
                            app.open_dialog("pageSetup", json!({}));
                        }
                    })
                    .response
                    .labelled_by(name.id);
            });
        }
        ui.horizontal(|ui| {
            ui.add_sized([44.0, 20.0], egui::Label::new(lang.tr("Scale:")));
            // The value is held while it is dragged or typed, so one change is one undo step.
            let held = ui.id().with("scale_to_fit_value");
            let mut value = ui.data(|d| d.get_temp::<u16>(held)).unwrap_or(scale);
            let drag = egui::DragValue::new(&mut value).range(10..=400).suffix("%");
            let r = ui.add_enabled(fit.is_none(), drag);
            if r.dragged() || r.has_focus() {
                ui.data_mut(|d| d.insert_temp(held, value));
            } else {
                ui.data_mut(|d| d.remove_temp::<u16>(held));
                if value != scale {
                    act(app, "pageLayout.scaleToFit", json!({"scale": value}));
                }
            }
            // Excel's dialog launcher: the corner arrow that opens Page Setup.
            let (corner, launcher) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::click());
            let stroke = egui::Stroke::new(1.0, if launcher.hovered() { t.accent } else { t.text_dim });
            let (a, b) = (corner.left_top() + vec2(3.0, 3.0), corner.right_bottom() - vec2(3.0, 3.0));
            for (from, to) in [(a, b), (b - vec2(5.0, 0.0), b), (b - vec2(0.0, 5.0), b)] {
                ui.painter().line_segment([from, to], stroke);
            }
            if launcher.on_hover_text(lang.tr("Page Setup")).clicked() {
                app.open_dialog("pageSetup", json!({}));
            }
        });
    });
}
