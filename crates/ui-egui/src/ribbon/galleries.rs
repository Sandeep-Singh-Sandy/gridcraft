use super::controls::{act, menu_items};
use super::{Align2, Cat, Icon, Rect, Sense, SheetApp, Stroke, StrokeKind, Tokens, Ui, icons, json, pos2, theme, vec2};

/// One Paste gallery tile: the Paste Special `what` it runs, a short badge drawn over the clipboard
/// (or brush) icon, and the tooltip. `Paste` itself is the plain `edit.paste`.
type PasteTile = (&'static str, &'static str, &'static str);

const PASTE_GROUPS: [(&str, &[PasteTile]); 3] = [
    (
        "Paste",
        &[
            ("all", "", "Paste (P)"),
            ("formulas", "fx", "Formulas (F)"),
            ("formulasAndNumberFormats", "%fx", "Formulas & Number Formatting (O)"),
            ("allUsingSourceTheme", "Aa", "Keep Source Formatting (K)"),
            ("allExceptBorders", "\u{2014}", "No Borders (B)"),
            ("keepColumnWidths", "\u{2194}", "Keep Source Column Widths (W)"),
            ("transpose", "T", "Transpose (T)"),
        ],
    ),
    (
        "Paste Values",
        &[
            ("values", "123", "Values (V)"),
            ("valuesAndNumberFormats", "%123", "Values & Number Formatting (A)"),
            ("valuesAndSourceFormatting", "A123", "Values & Source Formatting (E)"),
        ],
    ),
    ("Other Paste Options", &[("formats", "", "Formatting (R)"), ("link", "=", "Paste Link (N)")]),
];

/// Excel's Paste gallery, offered once something is copied: three groups of icon tiles with
/// tooltips, then Paste Special.
pub(super) fn paste_gallery(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    for (group, tiles) in PASTE_GROUPS {
        ui.label(egui::RichText::new(app.ui.language.tr(group)).strong());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for &(what, badge, tip) in tiles {
                let (r, resp) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 3.0, t.hover);
                    ui.painter().rect_stroke(r, 3.0, Stroke::new(1.0, t.accent), StrokeKind::Inside);
                }
                let icon = if group == "Other Paste Options" && what == "formats" { Icon::Brush } else { Icon::Paste };
                icons::paint(ui.painter(), Rect::from_center_size(r.center() - vec2(0.0, 2.0), vec2(22.0, 22.0)), icon, t.text);
                if !badge.is_empty() {
                    let corner = r.right_bottom() - vec2(3.0, 2.0);
                    ui.painter().text(corner, Align2::RIGHT_BOTTOM, badge, theme::ui_font(9.5), t.accent);
                }
                let tip = app.ui.language.tr(tip).to_string();
                if resp.on_hover_text(tip).clicked() {
                    match what {
                        "all" => app.paste_from_clipboard(),
                        "link" => act(app, "edit.pasteSpecial", json!({"what": "all", "link": true})),
                        "transpose" => act(app, "edit.pasteSpecial", json!({"what": "all", "transpose": true})),
                        "keepColumnWidths" => act(app, "edit.pasteSpecial", json!({"what": "all", "keepColumnWidths": true})),
                        what => act(app, "edit.pasteSpecial", json!({"what": what})),
                    }
                    ui.close();
                }
            }
        });
    }
    ui.separator();
    if ui.add(egui::Button::new(app.ui.language.tr("Paste Special…")).frame_when_inactive(false).min_size(vec2(220.0, 22.0))).clicked() {
        app.open_dialog("pasteSpecial", json!({}));
        ui.close();
    }
}

/// Page Layout › Margins as Excel lists it: a row per preset with its six margins in centimetres
/// and a page drawn with them, the sheet's current setting outlined, and "Last Custom Setting"
/// once margins were typed in Page Setup.
pub(super) fn margins_gallery(app: &mut SheetApp, ui: &mut Ui) {
    use gridcraft_engine::cmd::view::MARGIN_PRESETS;
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language;
    let same = |a: &[f32; 6], b: &[f32; 6]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.005);
    let current = app.session.active().and_then(|d| d.wb.active()).map(|sh| sh.print.margins);
    // Remembered for the session, so the entry stays after a preset replaces the custom margins.
    let memory = egui::Id::new("last_custom_margins");
    if let Some(m) = current.filter(|m| !MARGIN_PRESETS.iter().any(|(.., p)| same(m, p))) {
        ui.ctx().data_mut(|d| d.insert_temp(memory, m));
    }
    let custom: Option<[f32; 6]> = ui.ctx().data(|d| d.get_temp(memory));
    let mut rows: Vec<(&str, [f32; 6], serde_json::Value)> =
        MARGIN_PRESETS.iter().map(|(key, label, m)| (*label, *m, json!({"preset": key}))).collect();
    if let Some(m) = custom {
        let names = ["left", "right", "top", "bottom", "header", "footer"];
        rows.push(("Last Custom Setting", m, json!(names.iter().zip(m).map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>())));
    }
    ui.set_min_width(300.0);
    for (label, m, params) in rows {
        let (r, resp) = ui.allocate_exact_size(vec2(300.0, 60.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 3.0, t.hover);
        }
        if current.is_some_and(|c| same(&c, &m)) {
            ui.painter().rect_stroke(r.shrink(1.0), 3.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
        }
        // A letter page with the margins marked, drawn 2.7 points to the inch.
        let page = Rect::from_min_size(r.left_top() + vec2(10.0, 8.0), vec2(8.5 * 2.7, 11.0 * 2.7));
        ui.painter().rect_filled(page, 0.0, t.grid_bg);
        ui.painter().rect_stroke(page, 0.0, Stroke::new(1.0, t.text_dim), StrokeKind::Inside);
        let inner = Rect::from_min_max(page.min + vec2(m[0], m[2]) * 2.7, page.max - vec2(m[1], m[3]) * 2.7);
        ui.painter().rect_stroke(inner, 0.0, Stroke::new(1.0, t.accent), StrokeKind::Inside);
        let cm = |i: usize| format!("{:.2} cm", f64::from(m[i]) * 2.54);
        let text = |ui: &Ui, s: String, y: f32, bold: bool| {
            let font = if bold { theme::ui_bold(12.0) } else { theme::ui_font(11.0) };
            ui.painter().text(r.left_top() + vec2(48.0, y), Align2::LEFT_TOP, s, font, t.text);
        };
        text(ui, lang.tr(label).to_string(), 4.0, true);
        for (row, (a, ia, b, ib)) in [("Top:", 2, "Bottom:", 3), ("Left:", 0, "Right:", 1), ("Header:", 4, "Footer:", 5)].into_iter().enumerate() {
            text(ui, format!("{} {}   {} {}", lang.tr(a), cm(ia), lang.tr(b), cm(ib)), 20.0 + row as f32 * 13.0, false);
        }
        if resp.clicked() {
            act(app, "pageLayout.margins", params);
            ui.close();
        }
    }
    ui.separator();
    menu_items(app, ui, &[("Custom Margins…", "dialog:pageSetup", json!({}))]);
}

/// Papers Page Layout › Size lists: the name `pageLayout.size` takes and the label shown.
const PAPER_SIZES: [(&str, &str); 19] = [
    ("Letter", "Letter"),
    ("Tabloid", "Tabloid"),
    ("Legal", "Legal"),
    ("Statement", "Statement"),
    ("Executive", "Executive"),
    ("A3", "A3"),
    ("A4", "A4"),
    ("A5", "A5"),
    ("B4", "B4 (JIS)"),
    ("B5", "B5 (JIS)"),
    ("16K 195 x 270 mm", "16K"),
    ("16K 184 x 260 mm", "16K"),
    ("16K 197 x 273 mm", "16K"),
    ("Japanese Postcard", "Japanese Postcard"),
    ("Double Japan Postcard Rotated", "Double Japan Postcard Rotated"),
    ("Envelope #10", "Envelope #10"),
    ("Envelope Monarch", "Envelope Monarch"),
    ("Envelope B5", "Envelope B5"),
    ("Envelope C5", "Envelope C5"),
];

/// Page Layout › Size as Excel lists it: a scrolling row per paper with a page drawn to its
/// proportions, its name and its size in centimetres, the sheet's paper outlined, and
/// "More Paper Sizes…" at the end.
pub(super) fn size_gallery(app: &mut SheetApp, ui: &mut Ui) {
    use gridcraft_engine::cmd::print::paper_size;
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language;
    let current = app.session.active().and_then(|d| d.wb.active()).map(|sh| paper_size(&sh.print.paper));
    // Two decimals without trailing zeros: 21, 29.7, 27.94.
    let cm = |pt: f32| format!("{:.2}", f64::from(pt) * 2.54 / 72.0).trim_end_matches('0').trim_end_matches('.').to_string();
    ui.set_min_width(260.0);
    egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
        for (name, label) in PAPER_SIZES {
            let (w, h) = paper_size(name);
            let (r, resp) = ui.allocate_exact_size(vec2(260.0, 46.0), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(r, 3.0, t.hover);
            }
            if current == Some((w, h)) {
                ui.painter().rect_stroke(r.shrink(1.0), 3.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
            }
            let k = (28.0 / w).min(34.0 / h);
            let page = Rect::from_center_size(pos2(r.left() + 24.0, r.center().y), vec2(w * k, h * k));
            ui.painter().rect_filled(page, 0.0, t.grid_bg);
            ui.painter().rect_stroke(page, 0.0, Stroke::new(1.0, t.text_dim), StrokeKind::Inside);
            ui.painter().text(r.left_top() + vec2(52.0, 6.0), Align2::LEFT_TOP, lang.tr(label), theme::ui_bold(12.0), t.text);
            let size = format!("{} cm x {} cm", cm(w), cm(h));
            ui.painter().text(r.left_top() + vec2(52.0, 24.0), Align2::LEFT_TOP, size, theme::ui_font(11.0), t.text_dim);
            if resp.clicked() {
                act(app, "pageLayout.size", json!({"paper": name}));
                ui.close();
            }
        }
    });
    ui.separator();
    menu_items(app, ui, &[("More Paper Sizes…", "dialog:pageSetup", json!({}))]);
}

/// A gallery of table styles drawn as mini tables.
pub(super) fn table_gallery(app: &mut SheetApp, ui: &mut Ui, cmd: &str) {
    ui.set_width(430.0);
    let Some(wb) = app.session.active().map(|d| d.wb.clone()) else { return };
    for (fam, n) in [("Light", 21u32), ("Medium", 28), ("Dark", 11)] {
        ui.label(egui::RichText::new(fam).strong());
        egui::Grid::new(("tg", fam)).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
            for i in 1..=n {
                let name = format!("TableStyle{fam}{i}");
                let (r, resp) = ui.allocate_exact_size(vec2(54.0, 38.0), Sense::click());
                paint_table_swatch(ui, r, &wb, &name);
                if resp.hovered() {
                    ui.painter().rect_stroke(r.expand(1.5), 2.0, Stroke::new(2.0, Tokens::get(ui.ctx()).cat(Cat::Orange)), StrokeKind::Outside);
                }
                if resp.on_hover_text(&name).clicked() {
                    act(app, cmd, json!({"style": name}));
                    ui.close();
                }
                if i % 7 == 0 {
                    ui.end_row();
                }
            }
        });
    }
}

pub(super) fn paint_table_swatch(ui: &Ui, r: Rect, wb: &gridcraft_engine::model::Workbook, style: &str) {
    let lk = gridcraft_engine::tables::look(style);
    let res = |c: Option<gridcraft_engine::model::Color>| c.and_then(|c| c.resolve(&wb.theme)).map(theme::color32);
    let p = ui.painter();
    let t = Tokens::get(ui.ctx());
    p.rect_filled(r, 0.0, t.grid_bg);
    let rows = 5;
    let rh = r.height() / rows as f32;
    for i in 0..rows {
        let rr = Rect::from_min_size(pos2(r.left(), r.top() + i as f32 * rh), vec2(r.width(), rh));
        let fill = if i == 0 {
            res(lk.header_fill)
        } else if i % 2 == 1 {
            res(lk.band_fill)
        } else {
            res(lk.body_fill)
        };
        if let Some(f) = fill {
            p.rect_filled(rr, 0.0, f);
        }
        if let Some(l) = res(lk.line) {
            p.line_segment([rr.left_bottom(), rr.right_bottom()], Stroke::new(1.0, l));
        }
        for k in 0..3 {
            let x = rr.left() + 4.0 + k as f32 * (r.width() - 8.0) / 3.0;
            let col = if i == 0 { res(Some(lk.header_font)).unwrap_or(t.cell_text) } else { t.sheet_control_border };
            p.line_segment([pos2(x, rr.center().y), pos2(x + 10.0, rr.center().y)], Stroke::new(1.0, col));
        }
    }
    p.rect_stroke(r, 0.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);
}

pub(super) fn cell_style_gallery(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.set_width(520.0);
    let Some(wb) = app.session.active().map(|d| d.wb.clone()) else { return };
    let groups: &[(&str, &[&str])] = &[
        ("Good, Bad and Neutral", &["Normal", "Bad", "Good", "Neutral"]),
        ("Data and Model", &["Calculation", "Check Cell", "Explanatory Text", "Input", "Linked Cell", "Note", "Output", "Warning Text"]),
        ("Titles and Headings", &["Heading 1", "Heading 2", "Heading 3", "Heading 4", "Title", "Total"]),
        (
            "Themed Cell Styles",
            &[
                "20% - Accent1",
                "20% - Accent2",
                "20% - Accent3",
                "20% - Accent4",
                "20% - Accent5",
                "20% - Accent6",
                "40% - Accent1",
                "40% - Accent2",
                "40% - Accent3",
                "40% - Accent4",
                "40% - Accent5",
                "40% - Accent6",
                "60% - Accent1",
                "60% - Accent2",
                "60% - Accent3",
                "60% - Accent4",
                "60% - Accent5",
                "60% - Accent6",
                "Accent1",
                "Accent2",
                "Accent3",
                "Accent4",
                "Accent5",
                "Accent6",
            ],
        ),
        ("Number Format", &["Comma", "Comma [0]", "Currency", "Currency [0]", "Percent"]),
    ];
    for (g, names) in groups {
        ui.label(egui::RichText::new(*g).strong());
        egui::Grid::new(("csg", *g)).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
            for (i, n) in names.iter().enumerate() {
                let st = gridcraft_engine::cmd::format::builtin_cell_style(n, &wb.theme).unwrap_or_default();
                let (r, resp) = ui.allocate_exact_size(vec2(82.0, 24.0), Sense::click());
                let fill = st
                    .fill
                    .fg
                    .resolve(&wb.theme)
                    .filter(|_| st.fill.pattern != gridcraft_engine::model::PatternType::None)
                    .map(theme::color32)
                    .unwrap_or(t.grid_bg);
                ui.painter().rect_filled(r, 0.0, fill);
                ui.painter().rect_stroke(r, 0.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);
                if !st.borders.bottom.is_none() {
                    let c = st.borders.bottom.color.resolve(&wb.theme).map(theme::color32).unwrap_or(t.cell_text);
                    ui.painter().line_segment(
                        [r.left_bottom() + vec2(0.0, -1.5), r.right_bottom() + vec2(0.0, -1.5)],
                        Stroke::new(st.borders.bottom.style.width(), c),
                    );
                }
                let col = st.font.color.resolve(&wb.theme).map(theme::color32).unwrap_or(t.cell_text);
                let fam = theme::cell_family(&st.font.name, st.font.bold, st.font.italic);
                ui.painter().text(
                    r.left_center() + vec2(4.0, 0.0),
                    Align2::LEFT_CENTER,
                    *n,
                    egui::FontId::new((st.font.size).clamp(10.0, 15.0), fam),
                    col,
                );
                if resp.hovered() {
                    ui.painter().rect_stroke(r.expand(1.5), 2.0, Stroke::new(2.0, Tokens::get(ui.ctx()).cat(Cat::Orange)), StrokeKind::Outside);
                }
                if resp.clicked() {
                    act(app, "home.cellStyle", json!({"name": n}));
                    ui.close();
                }
                if (i + 1) % 6 == 0 {
                    ui.end_row();
                }
            }
        });
    }
}
