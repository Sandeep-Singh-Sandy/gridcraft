use super::{
    Align2, Backstage, Color32, Icon, Json, Language, Outcome, Rect, Sense, SheetApp, Stroke, StrokeKind, Tokens, Ui, dim, icons, json, pos2,
    section, square_button, theme, vec2,
};

use std::sync::Arc;

use egui::epaint::{PathShape, PathStroke};
use egui::{FontId, Painter, Pos2};
use gridcraft_engine::cmd::print::{DrawOp, Family as PdfFamily, PagePreview, PrintOptions, paper_size, preview_page};
use gridcraft_engine::model::Orientation;

use crate::printing::{Discover, Duplex, Feedback, State};

/// Width of the settings column; the preview takes the rest of the window.
const LEFT_W: f32 = 344.0;
/// Width and height of a setting's card, and the height of a row in its drop-down list.
const CARD_W: f32 = 332.0;
const CARD_H: f32 = 56.0;
const ROW_H: f32 = 52.0;

/// Papers the Print page offers: (name Page Setup takes, name shown).
const PAPERS: [(&str, &str); 10] = [
    ("Letter", "Letter"),
    ("Legal", "Legal"),
    ("A4", "A4"),
    ("A3", "A3"),
    ("A5", "A5"),
    ("A6", "A6"),
    ("Tabloid", "Tabloid"),
    ("Executive", "Executive"),
    ("B4", "B4 (JIS)"),
    ("B5", "B5 (JIS)"),
];

/// Margin presets in inches: (name, left, right, top, bottom, header, footer).
const MARGINS: [(&str, [f32; 6]); 4] = [
    ("Normal Margins", [0.7, 0.7, 0.75, 0.75, 0.3, 0.3]),
    ("Wide Margins", [1.0, 1.0, 1.0, 1.0, 0.5, 0.5]),
    ("Narrow Margins", [0.25, 0.25, 0.75, 0.75, 0.3, 0.3]),
    ("No Margins", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
];

/// Preview zoom choices: (name, percent of the paper's real size; 0 fits the page to the window).
const ZOOMS: [(&str, u16); 6] = [("Zoom to Page", 0), ("50%", 50), ("75%", 75), ("100%", 100), ("150%", 150), ("200%", 200)];

// ---------------------------------------------------------------- choice cards

/// One choice of a setting: an icon, a title and an optional second line.
#[derive(Clone)]
struct Choice {
    icon: Icon,
    title: String,
    detail: String,
    selected: bool,
    /// Marks the icon with a green tick (a printer that is ready).
    ready: bool,
}

impl Choice {
    fn new(icon: Icon, title: &str, detail: &str, selected: bool) -> Choice {
        Choice { icon, title: title.to_owned(), detail: detail.to_owned(), selected, ready: false }
    }

    fn ready(mut self, ready: bool) -> Choice {
        self.ready = ready;
        self
    }

    /// The choice marked selected, else `other`.
    fn current(choices: &[Choice], other: Choice) -> Choice {
        choices.iter().find(|c| c.selected).cloned().unwrap_or(other)
    }
}

/// `text` cut with an ellipsis so that it is at most `max_w` wide in `font`.
fn ellipsize(p: &Painter, text: &str, font: &FontId, max_w: f32, color: Color32) -> String {
    let width = |s: &str| p.layout_no_wrap(s.to_owned(), font.clone(), color).size().x;
    if width(text) <= max_w {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        let cut: String = chars.iter().take(mid).collect::<String>() + "…";
        if width(&cut) <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars.iter().take(lo).collect::<String>() + "…"
}

/// Paints a choice's icon, title and second line in `rect`, keeping `right` points free at its right.
fn paint_choice(p: &Painter, rect: Rect, choice: &Choice, t: &Tokens, right: f32) {
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 28.0, rect.center().y), vec2(28.0, 28.0));
    icons::paint(p, icon_rect, choice.icon, t.text);
    if choice.ready {
        // A green tick on the icon's lower right corner.
        let at = icon_rect.right_bottom() - vec2(3.0, 3.0);
        p.circle_filled(at, 7.0, Color32::from_rgb(0x2e, 0xa0, 0x43));
        icons::paint(p, Rect::from_center_size(at, vec2(10.0, 10.0)), Icon::Check, Color32::WHITE);
    }
    let x = rect.left() + 58.0;
    let max_w = (rect.right() - right - x).max(24.0);
    let (title_font, detail_font) = (theme::ui_font(14.0), theme::ui_font(12.0));
    let title = ellipsize(p, &choice.title, &title_font, max_w, t.text);
    if choice.detail.is_empty() {
        p.text(pos2(x, rect.center().y), Align2::LEFT_CENTER, title, title_font, t.text);
    } else {
        p.text(pos2(x, rect.center().y - 10.0), Align2::LEFT_CENTER, title, title_font, t.text);
        let detail = ellipsize(p, &choice.detail, &detail_font, max_w, t.text_dim);
        p.text(pos2(x, rect.center().y + 10.0), Align2::LEFT_CENTER, detail, detail_font, t.text_dim);
    }
}

/// A row of a setting's drop-down list.
fn choice_row(ui: &mut Ui, choice: &Choice, t: &Tokens) -> egui::Response {
    let width = ui.available_width().max(CARD_W - 16.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, choice.selected, &choice.title));
    let p = ui.painter();
    if choice.selected || resp.hovered() {
        p.rect_filled(rect, 3.0, if choice.selected { t.hover } else { t.hover.gamma_multiply(0.7) });
    }
    if choice.selected {
        p.rect_filled(Rect::from_min_size(rect.min, vec2(3.0, rect.height())), 1.0, t.accent);
        icons::paint(p, Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(14.0, 14.0)), Icon::Check, t.text);
    }
    paint_choice(p, rect, choice, t, 38.0);
    resp
}

/// A setting drawn as a bordered card (icon, title, second line) that opens a drop-down list of
/// `choices` with the same layout. `shown` is what the card says now. Returns the picked index.
fn select(ui: &mut Ui, id: &str, name: &str, shown: &Choice, choices: &[Choice]) -> Option<usize> {
    let t = Tokens::get(ui.ctx());
    let enabled = ui.is_enabled();
    let (_, rect) = ui.allocate_space(vec2(CARD_W, CARD_H));
    let resp = ui.interact(rect, ui.make_persistent_id(id), Sense::click());
    let label = format!("{name}: {}", shown.title);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &label));
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp));
    {
        let p = ui.painter();
        p.rect_filled(rect, 4.0, if resp.hovered() { t.hover } else { t.grid_bg });
        p.rect_stroke(rect, 4.0, Stroke::new(1.0, if resp.hovered() || open { t.accent } else { t.ribbon_border }), StrokeKind::Inside);
        paint_choice(p, rect, shown, &t, 36.0);
        icons::paint(p, Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(14.0, 14.0)), Icon::Chevron, t.text);
    }
    let mut picked = None;
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(CARD_W);
        ui.spacing_mut().item_spacing.y = 2.0;
        for (i, choice) in choices.iter().enumerate() {
            if choice_row(ui, choice, &t).clicked() {
                picked = Some(i);
            }
        }
    });
    picked
}

/// A compact drop-down (the preview's zoom).
fn choose(ui: &mut Ui, id: &str, name: &str, width: f32, shown: &str, entries: &[(String, bool)]) -> Option<usize> {
    let mut picked = None;
    craft_ui::ComboBox::from_id_salt(id)
        .width(width)
        .wrap_mode(egui::TextWrapMode::Truncate)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (i, (text, selected)) in entries.iter().enumerate() {
                if ui.selectable_label(*selected, text.as_str()).clicked() {
                    picked = Some(i);
                }
            }
        })
        .response
        .accessible_name(name);
    picked
}

// ---------------------------------------------------------------- settings

/// The active sheet's page setup, as far as this page shows it.
struct Current {
    landscape: bool,
    paper: String,
    margins: [f32; 6],
    scale: u16,
    fit: Option<(u16, u16)>,
}

fn current(app: &SheetApp) -> Option<Current> {
    let d = app.session.active()?;
    let p = &d.wb.sheet(d.wb.active_sheet())?.print;
    Some(Current { landscape: p.orientation == Orientation::Landscape, paper: p.paper.clone(), margins: p.margins, scale: p.scale, fit: p.fit_to })
}

/// A paper's size, as the card's second line.
fn paper_text(name: &str) -> String {
    let (w, h) = paper_size(name);
    format!("{:.1} cm × {:.1} cm", w / 72.0 * 2.54, h / 72.0 * 2.54)
}

/// A set of margins, as the card's second line.
fn margins_text(m: &[f32; 6]) -> String {
    let cm = |i: usize| m.get(i).copied().unwrap_or(0.0) * 2.54;
    format!("Top: {:.2} cm  Bottom: {:.2} cm  Left: {:.2} cm  Right: {:.2} cm", cm(2), cm(3), cm(0), cm(1))
}

/// The Scaling list: title, second line and the Page Setup parameters it sends (`Null` opens the
/// Page Setup dialog).
fn scaling_entries() -> Vec<(String, String, Json)> {
    let mut list = vec![
        ("No Scaling".to_owned(), "Print sheets at their actual size".to_owned(), json!({"scale": 100})),
        ("Fit Sheet on One Page".to_owned(), "Shrink the sheet so that it fits on one page".to_owned(), json!({"fitToPage": true})),
        ("Fit All Columns on One Page".to_owned(), "Shrink the sheet so that it is one page wide".to_owned(), json!({"fitWidth": 1, "fitHeight": 0})),
        ("Fit All Rows on One Page".to_owned(), "Shrink the sheet so that it is one page tall".to_owned(), json!({"fitWidth": 0, "fitHeight": 1})),
    ];
    for percent in [50, 75, 90, 110, 125] {
        list.push((format!("Scale to {percent}%"), format!("Print at {percent}% of the actual size"), json!({"scale": percent})));
    }
    list.push(("Custom Scaling…".to_owned(), "Set the scale in Page Setup".to_owned(), Json::Null));
    list
}

/// Which entry of [`scaling_entries`] the sheet's setup matches, if any.
fn scaling_index(c: &Current) -> Option<usize> {
    match c.fit {
        Some((1, 1)) => Some(1),
        Some((1, 0)) => Some(2),
        Some((0, 1)) => Some(3),
        Some(_) => None,
        None => match c.scale {
            100 => Some(0),
            50 => Some(4),
            75 => Some(5),
            90 => Some(6),
            110 => Some(7),
            125 => Some(8),
            _ => None,
        },
    }
}

/// Applies a Page Setup change from this page; the preview follows because the revision changes.
fn set_page(app: &mut SheetApp, bs: &mut Backstage, ctx: &egui::Context, params: Json) {
    if app.commit_edit(0, 0, false, false) {
        app.run_or_alert("pageLayout.pageSetup", params);
        bs.preview_page = 0;
        ctx.request_repaint();
    }
}

/// The page range typed under Pages for a printout of `total` pages: `Ok(None)` is every page,
/// `Err` an entry that is not a usable range.
fn page_range(bs: &Backstage, total: usize) -> Result<Option<(u32, u32)>, ()> {
    let parse = |text: &str| -> Result<Option<u32>, ()> {
        let text = text.trim();
        if text.is_empty() { Ok(None) } else { text.parse::<u32>().ok().filter(|n| *n > 0).map(Some).ok_or(()) }
    };
    let (from, to) = (parse(&bs.pages_from)?, parse(&bs.pages_to)?);
    if from.is_none() && to.is_none() {
        return Ok(None);
    }
    let total = u32::try_from(total).unwrap_or(u32::MAX);
    let (from, to) = (from.unwrap_or(1), to.unwrap_or(total).min(total));
    if from > to { Err(()) } else { Ok(Some((from, to))) }
}

/// The Print page (File › Print): settings on the left, the page preview filling the rest.
/// Caches `file.printPreview` per document revision and scope; page-setup choices go through
/// `pageLayout.pageSetup`, so they are saved with the workbook as in a spreadsheet's Page Setup.
pub(super) fn print(app: &mut SheetApp, ui: &mut Ui, bs: &mut Backstage, out: &mut Outcome) {
    let lang = app.ui.language;
    let Some((uid, revision)) = app.session.active().map(|d| (d.uid, d.revision)) else {
        dim(ui, lang.tr("No workbook is open."));
        return;
    };
    let params = json!({"sheets": if bs.print_all { "all" } else { "active" }});
    let key = (uid, revision, bs.print_all);
    if bs.preview.as_ref().is_none_or(|(k, _)| *k != key) {
        let preview = app.session.run("file.printPreview", params.clone()).unwrap_or(Json::Null);
        bs.preview = Some((key, preview));
    }
    let pages: Vec<Json> = bs.preview.as_ref().and_then(|(_, p)| p.get("pageList")).and_then(Json::as_array).cloned().unwrap_or_default();
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(LEFT_W);
            settings(app, ui, bs, out, &params, pages.len());
        });
        ui.add_space(24.0);
        ui.vertical(|ui| preview(app, ui, bs, &pages, (uid, revision)));
    });
}

/// The settings column: Print and Copies, the printer, then the sheet's page setup.
fn settings(app: &mut SheetApp, ui: &mut Ui, bs: &mut Backstage, out: &mut Outcome, params: &Json, total: usize) {
    let lang = app.ui.language;
    let range = page_range(bs, total);
    let can_print = (app.services.print_native.is_none() || app.printing.ready()) && range.is_ok() && total > 0;
    ui.horizontal_top(|ui| {
        if ui.add_enabled_ui(can_print, |ui| square_button(ui, Icon::Print, lang.tr("Print"))).inner.clicked() && app.commit_edit(0, 0, false, false)
        {
            let mut job = params.clone();
            if let Ok(Some((from, to))) = range {
                job["pageFrom"] = json!(from);
                job["pageTo"] = json!(to);
            }
            app.print_now(job);
            out.close = app.services.print_native.is_none();
        }
        ui.add_space(8.0);
        ui.vertical(|ui| {
            let label = ui.label(lang.tr("Copies"));
            ui.add(egui::DragValue::new(&mut app.printing.settings.copies).range(1..=999)).labelled_by(label.id);
        });
    });
    // What happened to the last job (the printer card says the rest).
    if let Some(message) = &app.printing.message
        && matches!(message, Feedback::Submitting | Feedback::Submitted(_) | Feedback::Failed(_) | Feedback::WorkerStopped)
    {
        dim(ui, &crate::printing::feedback(message, lang, |label| lang.tr(label).to_owned()));
    }

    section(ui, lang, "Printer");
    if let Some(discover) = &app.services.printers {
        crate::printing::prepare(&mut app.printing, ui.ctx(), discover);
        printer_controls(&mut app.printing, ui, discover, lang);
    }

    section(ui, lang, "Settings");
    let scope = [
        Choice::new(Icon::Sheet, lang.tr("Print Active Sheets"), lang.tr("Only print the active sheet"), !bs.print_all),
        Choice::new(Icon::Book, lang.tr("Print Entire Workbook"), lang.tr("Print every sheet of the workbook"), bs.print_all),
    ];
    let shown = Choice::current(&scope, scope[0].clone());
    if let Some(i) = select(ui, "backstage_print_scope", lang.tr("Print What"), &shown, &scope) {
        bs.print_all = i == 1;
        bs.preview_page = 0;
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let label = ui.label(lang.tr("Pages:"));
        ui.add(egui::TextEdit::singleline(&mut bs.pages_from).desired_width(52.0).hint_text("1")).labelled_by(label.id);
        ui.label(lang.tr("to"));
        ui.add(egui::TextEdit::singleline(&mut bs.pages_to).desired_width(52.0).hint_text(total.max(1).to_string()));
    });
    if range.is_err() {
        dim(ui, lang.tr("Enter a page range within the printout."));
    }
    ui.add_space(6.0);
    sides(&mut app.printing, ui, lang);

    // The sheet's page setup.
    let Some(c) = current(app) else { return };
    let ctx = ui.ctx().clone();
    ui.add_space(6.0);
    let orientation = [
        Choice::new(Icon::Portrait, lang.tr("Portrait Orientation"), "", !c.landscape),
        Choice::new(Icon::Landscape, lang.tr("Landscape Orientation"), "", c.landscape),
    ];
    let shown = Choice::current(&orientation, orientation[0].clone());
    if let Some(i) = select(ui, "backstage_orientation", lang.tr("Orientation"), &shown, &orientation) {
        set_page(app, bs, &ctx, json!({"orientation": if i == 1 { "landscape" } else { "portrait" }}));
    }

    ui.add_space(6.0);
    let papers: Vec<Choice> =
        PAPERS.iter().map(|(name, text)| Choice::new(Icon::Paper, text, &paper_text(name), name.eq_ignore_ascii_case(c.paper.trim()))).collect();
    let shown = Choice::current(&papers, Choice::new(Icon::Paper, &c.paper, &paper_text(&c.paper), false));
    if let Some(i) = select(ui, "backstage_paper", lang.tr("Paper Size"), &shown, &papers)
        && let Some((name, _)) = PAPERS.get(i)
    {
        set_page(app, bs, &ctx, json!({"paper": name}));
    }

    ui.add_space(6.0);
    let same = |preset: &[f32; 6]| preset.iter().zip(&c.margins).take(4).all(|(a, b)| (a - b).abs() < 0.005);
    let mut margins: Vec<Choice> =
        MARGINS.iter().map(|(name, preset)| Choice::new(Icon::Margins, lang.tr(name), &margins_text(preset), same(preset))).collect();
    margins.push(Choice::new(Icon::Margins, lang.tr("Custom Margins…"), lang.tr("Set the margins in Page Setup"), false));
    let shown = Choice::current(&margins, Choice::new(Icon::Margins, lang.tr("Custom Margins"), &margins_text(&c.margins), false));
    if let Some(i) = select(ui, "backstage_margins", lang.tr("Margins"), &shown, &margins) {
        match MARGINS.get(i) {
            Some((_, [left, right, top, bottom, header, footer])) => set_page(
                app,
                bs,
                &ctx,
                json!({"margins": {"left": left, "right": right, "top": top, "bottom": bottom, "header": header, "footer": footer}}),
            ),
            None => {
                app.open_dialog("pageSetup", json!({}));
            }
        }
    }

    ui.add_space(6.0);
    let scaling = scaling_entries();
    let on = scaling_index(&c);
    let entries: Vec<Choice> = scaling
        .iter()
        .enumerate()
        .map(|(i, (title, detail, _))| Choice::new(Icon::FitPage, lang.tr(title), lang.tr(detail), on == Some(i)))
        .collect();
    let custom = match c.fit {
        Some((w, h)) => lang.tr("Fit to {w} × {h} pages").replace("{w}", &w.to_string()).replace("{h}", &h.to_string()),
        None => lang.tr("Scale to {percent}%").replace("{percent}", &c.scale.to_string()),
    };
    let shown = Choice::current(&entries, Choice::new(Icon::FitPage, &custom, lang.tr("Set in Page Setup"), false));
    if let Some(i) = select(ui, "backstage_scaling", lang.tr("Scaling"), &shown, &entries)
        && let Some((_, _, change)) = scaling.get(i)
    {
        if change.is_null() {
            app.open_dialog("pageSetup", json!({}));
        } else {
            set_page(app, bs, &ctx, change.clone());
        }
    }
    ui.add_space(8.0);
    if ui.link(lang.tr("Page Setup…")).clicked() {
        app.open_dialog("pageSetup", json!({}));
    }
}

/// The printer card with its status, Refresh, and Grayscale.
fn printer_controls(state: &mut State, ui: &mut Ui, discover: &Discover, lang: Language) {
    ui.add_enabled_ui(!state.busy(), |ui| {
        let names = craft_ui::printing::shown_names(&state.printers);
        let status = if state.busy() {
            lang.tr("Submitting print job…")
        } else if state.discovering() {
            lang.tr("Finding printers…")
        } else if state.printers.is_empty() {
            lang.tr("No printers found")
        } else if state.ready() {
            lang.tr("Ready")
        } else {
            lang.tr("Not available")
        };
        let choices: Vec<Choice> = state
            .printers
            .iter()
            .zip(&names)
            .map(|(printer, text)| {
                Choice::new(Icon::Print, text, if printer.default { lang.tr("Default printer") } else { "" }, printer.name == state.settings.printer)
                    .ready(true)
            })
            .collect();
        let title = match choices.iter().find(|c| c.selected) {
            Some(chosen) => chosen.title.clone(),
            None if state.settings.printer.is_empty() => lang.tr("Select a printer").to_owned(),
            None => state.settings.printer.clone(),
        };
        let shown = Choice::new(Icon::Print, &title, status, false).ready(state.ready());
        if let Some(i) = select(ui, "backstage_printer", lang.tr("Printer"), &shown, &choices)
            && let Some(printer) = state.printers.get(i)
        {
            state.settings.printer = printer.name.clone();
        }
        ui.add_space(4.0);
        if ui.add_enabled(!state.discovering(), egui::Button::new(lang.tr("Refresh printers"))).clicked() {
            state.refresh(discover);
        }
        ui.checkbox(&mut state.settings.grayscale, lang.tr("Grayscale"));
    });
}

/// The two-sided card, which only the printer's job options carry.
fn sides(state: &mut State, ui: &mut Ui, lang: Language) {
    let options = [
        (None, "Printer default", "Use the printer's own two-sided setting"),
        (Some(Duplex::Off), "One-sided", "Only print on one side of the paper"),
        (Some(Duplex::LongEdge), "Two-sided: long edge", "Turn the pages over the long edge"),
        (Some(Duplex::ShortEdge), "Two-sided: short edge", "Turn the pages over the short edge"),
    ];
    let choices = options.map(|(value, title, detail)| Choice::new(Icon::Sides, lang.tr(title), lang.tr(detail), value == state.settings.duplex));
    let shown = Choice::current(&choices, choices[0].clone());
    ui.add_enabled_ui(!state.busy(), |ui| {
        if let Some(i) = select(ui, "backstage_duplex", lang.tr("Two-sided printing"), &shown, &choices)
            && let Some((value, _, _)) = options.get(i)
        {
            state.settings.duplex = *value;
        }
    });
}

// ---------------------------------------------------------------- preview

/// The page preview: the printed page at the chosen zoom, with page navigation below.
fn preview(app: &mut SheetApp, ui: &mut Ui, bs: &mut Backstage, pages: &[Json], (uid, revision): (u64, u64)) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language;
    if pages.is_empty() {
        dim(ui, lang.tr("Nothing to print."));
        return;
    }
    let last = pages.len().saturating_sub(1);
    bs.preview_page = bs.preview_page.min(last);
    let Some(page) = pages.get(bs.preview_page) else { return };
    // The page as the printout draws it (cached per document revision and page).
    let key = (uid, revision, bs.print_all, bs.preview_page);
    if bs.page_preview.as_ref().is_none_or(|(k, _)| *k != key) {
        let opts = PrintOptions { all_sheets: bs.print_all, ..PrintOptions::default() };
        let built = app.session.active().and_then(|d| {
            let path = d.path.clone().unwrap_or_default();
            preview_page(&d.wb, &opts, &d.display_title(), &path, bs.preview_page + 1)
        });
        bs.page_preview = Some((key, built.map(Arc::new)));
    }
    let drawn: Option<Arc<PagePreview>> = bs.page_preview.as_ref().and_then(|(_, p)| p.clone());
    // Paper proportions (points); hostile sizes fall back to Letter.
    let (w, h) = match &drawn {
        Some(d) => (d.width, d.height),
        None => (page.get("width").and_then(Json::as_f64).unwrap_or(612.0) as f32, page.get("height").and_then(Json::as_f64).unwrap_or(792.0) as f32),
    };
    let (w, h) = (if w.is_finite() && w > 1.0 { w } else { 612.0 }, if h.is_finite() && h > 1.0 { h } else { 792.0 });
    // The page fills the space the window leaves; a zoom shows the paper at that share of its real size.
    let room_w = ui.available_width().max(240.0);
    let room_h = (ui.clip_rect().bottom() - ui.next_widget_position().y - 64.0).clamp(260.0, 2400.0);
    // Room for the margins around the page and the spacing between them.
    let fit = ((room_h - 44.0) / h).min((room_w - 28.0) / w).max(0.05);
    let scale = if bs.preview_zoom == 0 { fit } else { f32::from(bs.preview_zoom) / 100.0 * 96.0 / 72.0 };
    let backdrop = if ui.visuals().dark_mode { Color32::from_gray(52) } else { Color32::from_gray(228) };
    ui.painter().rect_filled(Rect::from_min_size(ui.next_widget_position(), vec2(room_w, room_h)), 4.0, backdrop);
    egui::ScrollArea::both().id_salt("print_preview_scroll").max_width(room_w).max_height(room_h).auto_shrink([false, false]).show(ui, |ui| {
        let pad = ((room_w - w * scale) / 2.0).max(12.0);
        ui.horizontal_top(|ui| {
            ui.add_space(pad);
            ui.vertical(|ui| {
                ui.add_space(12.0);
                let (rect, _) = ui.allocate_exact_size(vec2(w * scale, h * scale), Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect.translate(vec2(2.0, 3.0)), 2.0, t.shadow);
                p.rect_filled(rect, 1.0, Color32::WHITE);
                if let Some(d) = &drawn {
                    paint_page(p, rect, d);
                }
                p.rect_stroke(rect, 1.0, Stroke::new(1.0, t.ribbon_border), StrokeKind::Inside);
                ui.add_space(12.0);
            });
        });
    });
    let sheet = page.get("sheet").and_then(Json::as_str).unwrap_or("");
    let range = page.get("range").and_then(Json::as_str).unwrap_or("");
    ui.add_space(4.0);
    dim(ui, &format!("{sheet} · {range}"));
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.add_enabled(bs.preview_page > 0, egui::Button::new("◀")).on_hover_text(lang.tr("Previous page")).clicked() {
            bs.preview_page = bs.preview_page.saturating_sub(1);
        }
        let mut number = bs.preview_page + 1;
        if ui.add(egui::DragValue::new(&mut number).range(1..=pages.len()).speed(0.05)).changed() {
            bs.preview_page = number.saturating_sub(1).min(last);
        }
        ui.label(lang.tr("of {pages}").replace("{pages}", &pages.len().to_string()));
        if ui.add_enabled(bs.preview_page < last, egui::Button::new("▶")).on_hover_text(lang.tr("Next page")).clicked() {
            bs.preview_page = (bs.preview_page + 1).min(last);
        }
        ui.add_space(24.0);
        let zooms: Vec<(String, bool)> = ZOOMS.iter().map(|(text, percent)| (lang.tr(text).to_owned(), *percent == bs.preview_zoom)).collect();
        let shown = zooms.iter().find(|(_, on)| *on).map(|(text, _)| text.clone()).unwrap_or_default();
        if let Some(i) = choose(ui, "backstage_zoom", lang.tr("Zoom"), 150.0, &shown, &zooms)
            && let Some((_, percent)) = ZOOMS.get(i)
        {
            bs.preview_zoom = *percent;
        }
    });
}

/// Replays the operations that draw a printed page, so the preview has the printout's fills,
/// borders, fonts, colours, wrapping, merged cells, conditional formats and charts. `rect` is the
/// paper on screen. Text is laid out in the sheet font family at the printed size and, where the
/// installed face is wider than the PDF's, shrunk to the width the printout gives it.
fn paint_page(p: &Painter, rect: Rect, page: &PagePreview) {
    let k = rect.width() / page.width.max(1.0);
    let ppp = p.ctx().pixels_per_point().max(1.0);
    let at = |x: f32, y: f32| pos2(rect.left() + x * k, rect.top() + y * k);
    let rgb = |c: [u8; 3]| Color32::from_rgb(c[0], c[1], c[2]);
    // A hairline stays visible; axis-aligned lines sit on the middle of a physical pixel.
    let line_w = |w: f32| (w * k).max(0.75);
    let snap = |v: f32| ((v * ppp).floor() + 0.5) / ppp;
    let mut clip = rect.intersect(p.clip_rect());
    let mut saved: Vec<Rect> = Vec::new();
    for op in &page.ops {
        match op {
            DrawOp::Save => saved.push(clip),
            DrawOp::Restore => {
                if let Some(before) = saved.pop() {
                    clip = before;
                }
            }
            DrawOp::Clip { x, y, w, h } => clip = clip.intersect(Rect::from_min_size(at(*x, *y), vec2(w * k, h * k))),
            _ if clip.width() <= 0.0 || clip.height() <= 0.0 => {}
            DrawOp::FillRect { x, y, w, h, color } => {
                // A hair wider than the cell, so that neighbouring fills leave no seam.
                let r = Rect::from_min_size(at(*x, *y), vec2(w * k + 0.4, h * k + 0.4));
                p.with_clip_rect(clip).rect_filled(r, 0.0, rgb(*color));
            }
            DrawOp::StrokeRect { x, y, w, h, width, color } => {
                let r = Rect::from_min_size(at(*x, *y), vec2(w * k, h * k));
                p.with_clip_rect(clip).rect_stroke(r, 0.0, Stroke::new(line_w(*width), rgb(*color)), StrokeKind::Middle);
            }
            DrawOp::Line { pts, width, color, dash } => {
                let mut v: Vec<Pos2> = pts.iter().map(|q| at(q[0], q[1])).collect();
                if v.len() < 2 {
                    continue;
                }
                let stroke = Stroke::new(line_w(*width), rgb(*color));
                let q = p.with_clip_rect(clip);
                if dash.is_empty() {
                    if let [a, b] = v.as_mut_slice() {
                        if (a.y - b.y).abs() < 0.01 {
                            (a.y, b.y) = (snap(a.y), snap(a.y));
                        } else if (a.x - b.x).abs() < 0.01 {
                            (a.x, b.x) = (snap(a.x), snap(a.x));
                        }
                    }
                    q.add(egui::Shape::line(v, stroke));
                } else {
                    let on = dash.first().copied().unwrap_or(3.0) * k;
                    let off = dash.get(1).copied().unwrap_or(2.0) * k;
                    q.extend(egui::Shape::dashed_line(&v, stroke, on.max(1.0), off.max(1.0)));
                }
            }
            DrawOp::Polygon { pts, fill, stroke } => {
                let v: Vec<Pos2> = pts.iter().map(|q| at(q[0], q[1])).collect();
                if v.len() < 3 {
                    continue;
                }
                let q = p.with_clip_rect(clip);
                if let Some(f) = fill {
                    if crate::chartview::is_convex(&v) {
                        q.add(PathShape { points: v.clone(), closed: true, fill: rgb(*f), stroke: PathStroke::NONE });
                    } else {
                        q.add(egui::Shape::mesh(crate::chartview::area_mesh(&v, rgb(*f))));
                    }
                }
                if let Some((c, w)) = stroke {
                    q.add(PathShape::closed_line(v, PathStroke::new(line_w(*w), rgb(*c))));
                }
            }
            DrawOp::Circle { cx, cy, r, fill, stroke } => {
                let (c, q) = (at(*cx, *cy), p.with_clip_rect(clip));
                if let Some(f) = fill {
                    q.circle_filled(c, r * k, rgb(*f));
                }
                if let Some((col, w)) = stroke {
                    q.circle_stroke(c, r * k, Stroke::new(line_w(*w), rgb(*col)));
                }
            }
            DrawOp::Text { x, y, size, font, color, text, angle, width } => {
                let px = size * k;
                if px < 1.5 || text.is_empty() {
                    continue;
                }
                let name = match font.family() {
                    PdfFamily::Sans => "Arial",
                    PdfFamily::Serif => "Times New Roman",
                    PdfFamily::Mono => "Courier New",
                };
                let family = theme::cell_family(name, font.is_bold(), font.is_italic());
                let (color, q) = (rgb(*color), p.with_clip_rect(clip));
                let mut galley = q.layout_no_wrap(text.clone(), FontId::new(px, family.clone()), color);
                let target = width * k;
                if target > 1.0 && galley.size().x > target * 1.03 {
                    galley = q.layout_no_wrap(text.clone(), FontId::new((px * target / galley.size().x).max(1.0), family), color);
                }
                // The origin is the baseline; the galley's top sits an ascent above it.
                let ascent = galley.size().y * 0.79;
                let origin = at(*x, *y);
                if angle.is_finite() && angle.abs() > f32::EPSILON {
                    let (s, c) = angle.sin_cos();
                    let mut shape = egui::epaint::TextShape::new(origin + vec2(ascent * s, -ascent * c), galley, color);
                    shape.angle = *angle;
                    q.add(shape);
                } else {
                    q.galley(origin - vec2(0.0, ascent), galley, color);
                }
            }
            DrawOp::Image { x, y, w, h } => {
                let r = Rect::from_min_size(at(*x, *y), vec2(w * k, h * k));
                let q = p.with_clip_rect(clip);
                q.rect_filled(r, 0.0, Color32::from_gray(238));
                q.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_gray(190)), StrokeKind::Inside);
                if r.width() > 30.0 && r.height() > 30.0 {
                    icons::paint_on_sheet(&q, Rect::from_center_size(r.center(), vec2(24.0, 24.0)), Icon::Picture, Color32::from_gray(120));
                }
            }
        }
    }
}
