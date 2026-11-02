use super::controls::act;
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
                    ui.painter().rect(r, 3.0, t.hover, Stroke::new(1.0, t.accent), StrokeKind::Inside);
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
