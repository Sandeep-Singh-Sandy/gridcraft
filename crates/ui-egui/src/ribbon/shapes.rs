//! The Insert › Shapes gallery: Excel's categories with every shape drawn in miniature.
//! Display only for now: the tiles are greyed out and run no command.
use super::{Rect, Sense, SheetApp, Stroke, StrokeKind, Tokens, Ui, pos2, vec2};
use egui::{Painter, Pos2, Shape};
use std::f32::consts::{FRAC_PI_2, TAU};

/// A drawing primitive on a 10x10 grid (x right, y down).
enum G {
    /// Rectangle x0, y0, x1, y1 with corner radius.
    Bx(u8, u8, u8, u8, u8),
    /// Ellipse inside x0, y0, x1, y1.
    Ov(u8, u8, u8, u8),
    /// Closed outline.
    Poly(&'static [(u8, u8)]),
    /// Open path; bit 0 puts an arrowhead at its end, bit 1 at its start.
    Open(&'static [(u8, u8)], u8),
    /// Cubic curve with the same arrowhead bits.
    Bez([(u8, u8); 4], u8),
    /// Circle arc from, to (degrees clockwise from 3 o'clock); 0 open, 1 pie, 2 chord.
    Arc(i16, i16, u8),
    /// Star with this many points and inner radius in percent.
    Star(u8, u8),
    /// Regular polygon with this many sides.
    Reg(u8),
}
use G::{Arc, Bez, Bx, Open, Ov, Poly, Reg, Star};

/// Tooltip, primitives, and a transform (bits 0-1: quarter turns clockwise, bit 2: mirror).
type Shp = (&'static str, &'static [G], u8);

const LINES: &[Shp] = &[
    ("Line", &[Open(&[(1, 9), (9, 1)], 0)], 0),
    ("Line Arrow", &[Open(&[(1, 9), (9, 1)], 1)], 0),
    ("Line Arrow: Double", &[Open(&[(1, 9), (9, 1)], 3)], 0),
    ("Connector: Elbow", &[Open(&[(1, 8), (5, 8), (5, 2), (9, 2)], 0)], 0),
    ("Connector: Elbow Arrow", &[Open(&[(1, 8), (5, 8), (5, 2), (9, 2)], 1)], 0),
    ("Connector: Elbow Double-Arrow", &[Open(&[(1, 8), (5, 8), (5, 2), (9, 2)], 3)], 0),
    ("Connector: Curved", &[Bez([(1, 8), (7, 8), (3, 2), (9, 2)], 0)], 0),
    ("Connector: Curved Arrow", &[Bez([(1, 8), (7, 8), (3, 2), (9, 2)], 1)], 0),
    ("Connector: Curved Double-Arrow", &[Bez([(1, 8), (7, 8), (3, 2), (9, 2)], 3)], 0),
    ("Curve", &[Bez([(1, 8), (1, 1), (9, 1), (9, 8)], 0)], 0),
    ("Freeform: Shape", &[Poly(&[(1, 8), (3, 2), (6, 6), (9, 1), (8, 9)])], 0),
    ("Freeform: Scribble", &[Open(&[(1, 7), (2, 3), (4, 8), (5, 2), (7, 8), (9, 3)], 0)], 0),
];

const RECTANGLES: &[Shp] = &[
    ("Rectangle", &[Bx(1, 2, 9, 8, 0)], 0),
    ("Rectangle: Rounded Corners", &[Bx(1, 2, 9, 8, 2)], 0),
    ("Rectangle: Single Corner Snipped", &[Poly(&[(1, 2), (7, 2), (9, 4), (9, 8), (1, 8)])], 0),
    ("Rectangle: Top Corners Snipped", &[Poly(&[(3, 2), (7, 2), (9, 4), (9, 8), (1, 8), (1, 4)])], 0),
    ("Rectangle: Diagonal Corners Snipped", &[Poly(&[(3, 2), (9, 2), (9, 6), (7, 8), (1, 8), (1, 4)])], 0),
    ("Rectangle: Snipped and Rounded Single Corner", &[Poly(&[(2, 2), (7, 2), (9, 4), (9, 8), (1, 8), (1, 3)])], 0),
    ("Rectangle: Single Corner Rounded", &[Poly(&[(1, 2), (8, 2), (9, 3), (9, 8), (1, 8)])], 0),
    ("Rectangle: Top Corners Rounded", &[Poly(&[(2, 2), (8, 2), (9, 3), (9, 8), (1, 8), (1, 3)])], 0),
    ("Rectangle: Diagonal Corners Rounded", &[Poly(&[(2, 2), (9, 2), (9, 7), (8, 8), (1, 8), (1, 3)])], 0),
];

const BASIC: &[Shp] = &[
    ("Text Box", &[Bx(1, 2, 9, 8, 0), Open(&[(3, 6), (5, 3), (7, 6)], 0), Open(&[(4, 5), (6, 5)], 0)], 0),
    ("Oval", &[Ov(1, 2, 9, 8)], 0),
    ("Triangle: Isosceles", &[Poly(&[(5, 1), (9, 9), (1, 9)])], 0),
    ("Triangle: Right", &[Poly(&[(1, 1), (9, 9), (1, 9)])], 0),
    ("Parallelogram", &[Poly(&[(3, 2), (9, 2), (7, 8), (1, 8)])], 0),
    ("Trapezoid", &[Poly(&[(3, 2), (7, 2), (9, 8), (1, 8)])], 0),
    ("Diamond", &[Poly(&[(5, 0), (10, 5), (5, 10), (0, 5)])], 0),
    ("Regular Pentagon", &[Reg(5)], 0),
    ("Hexagon", &[Reg(6)], 0),
    ("Heptagon", &[Reg(7)], 0),
    ("Octagon", &[Reg(8)], 0),
    ("Decagon", &[Reg(10)], 0),
    ("Dodecagon", &[Reg(12)], 0),
    ("Pie", &[Arc(0, 270, 1)], 0),
    ("Chord", &[Arc(45, 315, 2)], 0),
    ("Teardrop", &[Arc(0, 270, 0), Open(&[(10, 5), (10, 0), (5, 0)], 0)], 0),
    ("Frame", &[Bx(1, 1, 9, 9, 0), Bx(3, 3, 7, 7, 0)], 0),
    ("Half Frame", &[Poly(&[(1, 1), (9, 1), (7, 3), (3, 3), (3, 9), (1, 9)])], 0),
    ("L-Shape", &[Poly(&[(2, 1), (5, 1), (5, 6), (9, 6), (9, 9), (2, 9)])], 0),
    ("Diagonal Stripe", &[Poly(&[(0, 10), (0, 5), (5, 0), (10, 0)])], 0),
    ("Cross", &[Poly(&[(3, 1), (7, 1), (7, 3), (9, 3), (9, 7), (7, 7), (7, 9), (3, 9), (3, 7), (1, 7), (1, 3), (3, 3)])], 0),
    ("Plaque", &[Bx(1, 2, 9, 8, 3)], 0),
    ("Cylinder", &[Ov(2, 0, 8, 3), Open(&[(2, 2), (2, 8)], 0), Open(&[(8, 2), (8, 8)], 0), Bez([(2, 8), (2, 11), (8, 11), (8, 8)], 0)], 0),
    ("Cube", &[Bx(1, 3, 7, 9, 0), Poly(&[(1, 3), (3, 1), (9, 1), (7, 3)]), Open(&[(7, 9), (9, 7), (9, 1)], 0)], 0),
    (
        "Bevel",
        &[
            Bx(1, 1, 9, 9, 0),
            Bx(3, 3, 7, 7, 0),
            Open(&[(1, 1), (3, 3)], 0),
            Open(&[(9, 1), (7, 3)], 0),
            Open(&[(1, 9), (3, 7)], 0),
            Open(&[(9, 9), (7, 7)], 0),
        ],
        0,
    ),
    ("Donut", &[Ov(0, 0, 10, 10), Ov(3, 3, 7, 7)], 0),
    ("\"No\" Symbol", &[Ov(0, 0, 10, 10), Ov(2, 2, 8, 8), Open(&[(2, 8), (8, 2)], 0)], 0),
    ("Block Arc", &[Arc(180, 360, 0), Open(&[(0, 5), (2, 5)], 0), Open(&[(8, 5), (10, 5)], 0), Bez([(2, 5), (2, 0), (8, 0), (8, 5)], 0)], 0),
    ("Folded Corner", &[Poly(&[(1, 1), (9, 1), (9, 6), (6, 9), (1, 9)]), Open(&[(6, 9), (6, 6), (9, 6)], 0)], 0),
    ("Smiley Face", &[Ov(0, 0, 10, 10), Ov(3, 3, 4, 4), Ov(6, 3, 7, 4), Bez([(2, 6), (3, 9), (7, 9), (8, 6)], 0)], 0),
    ("Heart", &[Poly(&[(5, 9), (1, 5), (1, 3), (3, 1), (5, 3), (7, 1), (9, 3), (9, 5)])], 0),
    ("Lightning Bolt", &[Poly(&[(2, 0), (6, 0), (4, 4), (8, 4), (3, 10), (5, 6), (2, 6)])], 0),
    (
        "Sun",
        &[
            Ov(3, 3, 7, 7),
            Open(&[(5, 0), (5, 2)], 0),
            Open(&[(5, 8), (5, 10)], 0),
            Open(&[(0, 5), (2, 5)], 0),
            Open(&[(8, 5), (10, 5)], 0),
            Open(&[(1, 1), (2, 2)], 0),
            Open(&[(9, 1), (8, 2)], 0),
            Open(&[(1, 9), (2, 8)], 0),
            Open(&[(9, 9), (8, 8)], 0),
        ],
        0,
    ),
    ("Moon", &[Arc(90, 270, 0), Bez([(5, 0), (2, 3), (2, 7), (5, 10)], 0)], 0),
    ("Cloud", &[Poly(&[(2, 8), (1, 6), (2, 4), (4, 4), (5, 2), (7, 2), (8, 4), (9, 5), (9, 7), (8, 8)])], 0),
    ("Arc", &[Arc(270, 360, 0)], 0),
    ("Double Bracket", &[Open(&[(3, 1), (1, 1), (1, 9), (3, 9)], 0), Open(&[(7, 1), (9, 1), (9, 9), (7, 9)], 0)], 0),
    (
        "Double Brace",
        &[Open(&[(4, 1), (2, 2), (2, 4), (1, 5), (2, 6), (2, 8), (4, 9)], 0), Open(&[(6, 1), (8, 2), (8, 4), (9, 5), (8, 6), (8, 8), (6, 9)], 0)],
        0,
    ),
    ("Left Bracket", &[Open(&[(7, 1), (3, 1), (3, 9), (7, 9)], 0)], 0),
    ("Right Bracket", &[Open(&[(7, 1), (3, 1), (3, 9), (7, 9)], 0)], 4),
    ("Left Brace", &[Open(&[(7, 1), (5, 2), (5, 4), (3, 5), (5, 6), (5, 8), (7, 9)], 0)], 0),
    ("Right Brace", &[Open(&[(7, 1), (5, 2), (5, 4), (3, 5), (5, 6), (5, 8), (7, 9)], 0)], 4),
];

const RIGHT: G = Poly(&[(0, 3), (6, 3), (6, 0), (10, 5), (6, 10), (6, 7), (0, 7)]);
const LEFT_RIGHT: G = Poly(&[(0, 5), (3, 2), (3, 4), (7, 4), (7, 2), (10, 5), (7, 8), (7, 6), (3, 6), (3, 8)]);
const CURVED: G = Poly(&[(0, 0), (5, 0), (7, 2), (7, 5), (10, 5), (6, 10), (3, 5), (5, 5), (5, 3), (4, 2), (0, 2)]);
const RIGHT_CALLOUT: [G; 2] = [Bx(0, 2, 5, 8, 0), Poly(&[(5, 4), (7, 4), (7, 2), (10, 5), (7, 8), (7, 6), (5, 6)])];

const BLOCK_ARROWS: &[Shp] = &[
    ("Arrow: Right", &[RIGHT], 0),
    ("Arrow: Left", &[RIGHT], 2),
    ("Arrow: Up", &[RIGHT], 3),
    ("Arrow: Down", &[RIGHT], 1),
    ("Arrow: Left-Right", &[LEFT_RIGHT], 0),
    ("Arrow: Up-Down", &[LEFT_RIGHT], 1),
    (
        "Arrow: Four Directions",
        &[Poly(&[
            (5, 0),
            (7, 2),
            (6, 2),
            (6, 4),
            (8, 4),
            (8, 3),
            (10, 5),
            (8, 7),
            (8, 6),
            (6, 6),
            (6, 8),
            (7, 8),
            (5, 10),
            (3, 8),
            (4, 8),
            (4, 6),
            (2, 6),
            (2, 7),
            (0, 5),
            (2, 3),
            (2, 4),
            (4, 4),
            (4, 2),
            (3, 2),
        ])],
        0,
    ),
    (
        "Arrow: Left-Right-Up",
        &[Poly(&[
            (5, 0),
            (7, 2),
            (6, 2),
            (6, 5),
            (8, 5),
            (8, 3),
            (10, 6),
            (8, 9),
            (8, 7),
            (2, 7),
            (2, 9),
            (0, 6),
            (2, 3),
            (2, 5),
            (4, 5),
            (4, 2),
            (3, 2),
        ])],
        0,
    ),
    ("Arrow: Bent", &[Poly(&[(0, 10), (0, 5), (2, 3), (6, 3), (6, 1), (10, 4), (6, 7), (6, 5), (3, 5), (3, 10)])], 0),
    (
        "Arrow: U-Turn",
        &[Poly(&[(0, 10), (0, 4), (2, 1), (6, 1), (8, 3), (8, 5), (10, 5), (7, 9), (4, 5), (6, 5), (6, 4), (5, 3), (3, 3), (2, 4), (2, 10)])],
        0,
    ),
    ("Arrow: Left-Up", &[Poly(&[(0, 7), (3, 4), (3, 6), (6, 6), (6, 3), (4, 3), (7, 0), (10, 3), (8, 3), (8, 8), (3, 8), (3, 10)])], 0),
    ("Arrow: Bent-Up", &[Poly(&[(0, 6), (6, 6), (6, 3), (4, 3), (7, 0), (10, 3), (8, 3), (8, 9), (0, 9)])], 0),
    ("Arrow: Curved Right", &[CURVED], 0),
    ("Arrow: Curved Left", &[CURVED], 2),
    ("Arrow: Curved Up", &[CURVED], 3),
    ("Arrow: Curved Down", &[CURVED], 1),
    ("Arrow: Striped Right", &[Poly(&[(4, 3), (6, 3), (6, 0), (10, 5), (6, 10), (6, 7), (4, 7)]), Bx(0, 3, 1, 7, 0), Bx(2, 3, 3, 7, 0)], 0),
    ("Arrow: Notched Right", &[Poly(&[(0, 3), (6, 3), (6, 0), (10, 5), (6, 10), (6, 7), (0, 7), (2, 5)])], 0),
    ("Arrow: Pentagon", &[Poly(&[(0, 1), (7, 1), (10, 5), (7, 9), (0, 9)])], 0),
    ("Arrow: Chevron", &[Poly(&[(0, 1), (6, 1), (10, 5), (6, 9), (0, 9), (4, 5)])], 0),
    ("Arrow: Right Callout", &RIGHT_CALLOUT, 0),
    ("Arrow: Left Callout", &RIGHT_CALLOUT, 2),
    ("Arrow: Up Callout", &RIGHT_CALLOUT, 3),
    ("Arrow: Down Callout", &RIGHT_CALLOUT, 1),
    ("Arrow: Circular", &[Arc(180, 360, 0), Poly(&[(7, 5), (10, 5), (9, 8)])], 0),
];

const EQUATION: &[Shp] = &[
    ("Equation: Plus", &[Poly(&[(4, 1), (6, 1), (6, 4), (9, 4), (9, 6), (6, 6), (6, 9), (4, 9), (4, 6), (1, 6), (1, 4), (4, 4)])], 0),
    ("Equation: Minus", &[Bx(1, 4, 9, 6, 0)], 0),
    ("Equation: Multiply", &[Poly(&[(2, 1), (5, 4), (8, 1), (9, 2), (6, 5), (9, 8), (8, 9), (5, 6), (2, 9), (1, 8), (4, 5), (1, 2)])], 0),
    ("Equation: Divide", &[Bx(1, 4, 9, 6, 0), Ov(4, 0, 6, 2), Ov(4, 8, 6, 10)], 0),
    ("Equation: Equal", &[Bx(1, 2, 9, 4, 0), Bx(1, 6, 9, 8, 0)], 0),
    ("Equation: Not Equal", &[Bx(1, 2, 9, 4, 0), Bx(1, 6, 9, 8, 0), Open(&[(3, 10), (7, 0)], 0)], 0),
];

const FLOWCHART: &[Shp] = &[
    ("Flowchart: Process", &[Bx(1, 2, 9, 8, 0)], 0),
    ("Flowchart: Alternate Process", &[Bx(1, 2, 9, 8, 2)], 0),
    ("Flowchart: Decision", &[Poly(&[(5, 1), (10, 5), (5, 9), (0, 5)])], 0),
    ("Flowchart: Data", &[Poly(&[(2, 2), (10, 2), (8, 8), (0, 8)])], 0),
    ("Flowchart: Predefined Process", &[Bx(1, 2, 9, 8, 0), Open(&[(2, 2), (2, 8)], 0), Open(&[(8, 2), (8, 8)], 0)], 0),
    ("Flowchart: Internal Storage", &[Bx(1, 2, 9, 8, 0), Open(&[(3, 2), (3, 8)], 0), Open(&[(1, 4), (9, 4)], 0)], 0),
    ("Flowchart: Document", &[Poly(&[(1, 1), (9, 1), (9, 8), (7, 7), (3, 9), (1, 8)])], 0),
    ("Flowchart: Multidocument", &[Open(&[(3, 1), (10, 1), (10, 7)], 0), Poly(&[(0, 3), (8, 3), (8, 9), (5, 8), (0, 9)])], 0),
    ("Flowchart: Terminator", &[Bx(0, 2, 10, 8, 3)], 0),
    ("Flowchart: Preparation", &[Poly(&[(2, 2), (8, 2), (10, 5), (8, 8), (2, 8), (0, 5)])], 0),
    ("Flowchart: Manual Input", &[Poly(&[(0, 3), (10, 1), (10, 9), (0, 9)])], 0),
    ("Flowchart: Manual Operation", &[Poly(&[(0, 2), (10, 2), (8, 8), (2, 8)])], 0),
    ("Flowchart: Connector", &[Ov(1, 1, 9, 9)], 0),
    ("Flowchart: Off-page Connector", &[Poly(&[(1, 1), (9, 1), (9, 6), (5, 9), (1, 6)])], 0),
    ("Flowchart: Card", &[Poly(&[(3, 1), (9, 1), (9, 9), (1, 9), (1, 3)])], 0),
    ("Flowchart: Punched Tape", &[Poly(&[(0, 2), (2, 1), (5, 2), (8, 3), (10, 2), (10, 8), (8, 9), (5, 8), (2, 7), (0, 8)])], 0),
    ("Flowchart: Summing Junction", &[Ov(1, 1, 9, 9), Open(&[(2, 2), (8, 8)], 0), Open(&[(8, 2), (2, 8)], 0)], 0),
    ("Flowchart: Or", &[Ov(1, 1, 9, 9), Open(&[(5, 1), (5, 9)], 0), Open(&[(1, 5), (9, 5)], 0)], 0),
    ("Flowchart: Collate", &[Poly(&[(1, 1), (9, 1), (1, 9), (9, 9)])], 0),
    ("Flowchart: Sort", &[Poly(&[(5, 0), (10, 5), (5, 10), (0, 5)]), Open(&[(0, 5), (10, 5)], 0)], 0),
    ("Flowchart: Extract", &[Poly(&[(5, 1), (9, 9), (1, 9)])], 0),
    ("Flowchart: Merge", &[Poly(&[(1, 1), (9, 1), (5, 9)])], 0),
    ("Flowchart: Stored Data", &[Poly(&[(2, 1), (10, 1), (8, 5), (10, 9), (2, 9), (0, 5)])], 0),
    ("Flowchart: Delay", &[Poly(&[(1, 1), (6, 1), (9, 3), (9, 7), (6, 9), (1, 9)])], 0),
    ("Flowchart: Sequential Access Storage", &[Ov(1, 1, 9, 9), Open(&[(6, 9), (9, 9)], 0)], 0),
    (
        "Flowchart: Magnetic Disk",
        &[Ov(2, 0, 8, 3), Open(&[(2, 2), (2, 8)], 0), Open(&[(8, 2), (8, 8)], 0), Bez([(2, 8), (2, 11), (8, 11), (8, 8)], 0)],
        0,
    ),
    (
        "Flowchart: Direct Access Storage",
        &[Ov(2, 0, 8, 3), Open(&[(2, 2), (2, 8)], 0), Open(&[(8, 2), (8, 8)], 0), Bez([(2, 8), (2, 11), (8, 11), (8, 8)], 0)],
        1,
    ),
    ("Flowchart: Display", &[Poly(&[(0, 5), (2, 1), (8, 1), (10, 5), (8, 9), (2, 9)])], 0),
];

const RIBBON: G = Poly(&[(0, 2), (2, 2), (2, 1), (8, 1), (8, 2), (10, 2), (8, 5), (10, 8), (8, 8), (8, 9), (2, 9), (2, 8), (0, 8), (2, 5)]);
const CURVED_RIBBON: G = Poly(&[(0, 2), (3, 3), (7, 3), (10, 2), (8, 5), (10, 8), (7, 7), (3, 7), (0, 8), (2, 5)]);
const SCROLL: [G; 3] = [Bx(2, 1, 8, 9, 0), Ov(0, 1, 2, 9), Ov(8, 1, 10, 9)];

const STARS: &[Shp] = &[
    ("Star: 4 Points", &[Star(4, 40)], 0),
    ("Star: 5 Points", &[Star(5, 38)], 0),
    ("Star: 6 Points", &[Star(6, 55)], 0),
    ("Star: 7 Points", &[Star(7, 60)], 0),
    ("Star: 8 Points", &[Star(8, 70)], 0),
    ("Star: 10 Points", &[Star(10, 70)], 0),
    ("Star: 12 Points", &[Star(12, 76)], 0),
    ("Star: 16 Points", &[Star(16, 80)], 0),
    ("Star: 24 Points", &[Star(24, 85)], 0),
    ("Star: 32 Points", &[Star(32, 88)], 0),
    ("Ribbon: Tilted Up", &[RIBBON], 0),
    ("Ribbon: Tilted Down", &[RIBBON], 2),
    ("Ribbon: Curved and Tilted Up", &[CURVED_RIBBON], 0),
    ("Ribbon: Curved and Tilted Down", &[CURVED_RIBBON], 2),
    (
        "Explosion 1",
        &[Poly(&[
            (5, 3),
            (6, 0),
            (7, 3),
            (9, 2),
            (8, 4),
            (10, 5),
            (8, 6),
            (9, 9),
            (6, 7),
            (5, 10),
            (4, 7),
            (1, 9),
            (2, 6),
            (0, 5),
            (2, 4),
            (1, 2),
            (4, 3),
        ])],
        0,
    ),
    ("Explosion 2", &[Star(14, 50)], 0),
    ("Scroll: Vertical", &SCROLL, 0),
    ("Scroll: Horizontal", &SCROLL, 1),
    ("Wave", &[Poly(&[(0, 3), (2, 2), (4, 3), (6, 4), (8, 3), (10, 2), (10, 7), (8, 8), (6, 9), (4, 8), (2, 7), (0, 8)])], 0),
    (
        "Double Wave",
        &[Poly(&[
            (0, 3),
            (1, 2),
            (2, 2),
            (3, 3),
            (5, 4),
            (7, 3),
            (8, 2),
            (9, 2),
            (10, 3),
            (10, 8),
            (9, 9),
            (8, 9),
            (7, 8),
            (5, 7),
            (3, 8),
            (2, 9),
            (1, 9),
            (0, 8),
        ])],
        0,
    ),
];

const CALLOUTS: &[Shp] = &[
    ("Speech Bubble: Rectangle", &[Poly(&[(0, 1), (10, 1), (10, 7), (5, 7), (3, 10), (3, 7), (0, 7)])], 0),
    ("Speech Bubble: Rectangle with Corners Rounded", &[Bx(0, 1, 10, 7, 2), Poly(&[(2, 7), (1, 10), (5, 7)])], 0),
    ("Speech Bubble: Oval", &[Ov(0, 0, 10, 7), Poly(&[(2, 6), (1, 10), (4, 7)])], 0),
    (
        "Thought Bubble: Cloud",
        &[Poly(&[(2, 6), (1, 4), (2, 2), (4, 2), (5, 0), (7, 0), (8, 2), (9, 3), (9, 5), (8, 6)]), Ov(2, 8, 3, 9), Ov(0, 9, 1, 10)],
        0,
    ),
    ("Callout: Line", &[Bx(3, 1, 10, 7, 0), Open(&[(0, 9), (3, 4)], 0)], 0),
    ("Callout: Bent Line", &[Bx(3, 1, 10, 7, 0), Open(&[(0, 9), (1, 6), (3, 4)], 0)], 0),
    ("Callout: Double Bent Line", &[Bx(3, 1, 10, 7, 0), Open(&[(0, 9), (1, 9), (2, 6), (3, 4)], 0)], 0),
    ("Callout: Line with Accent Bar", &[Bx(3, 1, 10, 7, 0), Open(&[(5, 1), (5, 7)], 0), Open(&[(0, 9), (3, 4)], 0)], 0),
    ("Callout: Bent Line with Accent Bar", &[Bx(3, 1, 10, 7, 0), Open(&[(5, 1), (5, 7)], 0), Open(&[(0, 9), (1, 6), (3, 4)], 0)], 0),
    ("Callout: Double Bent Line with Accent Bar", &[Bx(3, 1, 10, 7, 0), Open(&[(5, 1), (5, 7)], 0), Open(&[(0, 9), (1, 9), (2, 6), (3, 4)], 0)], 0),
    ("Callout: Line with No Border", &[Open(&[(5, 3), (9, 3)], 0), Open(&[(5, 5), (9, 5)], 0), Open(&[(0, 9), (3, 4)], 0)], 0),
    ("Callout: Bent Line with No Border", &[Open(&[(5, 3), (9, 3)], 0), Open(&[(5, 5), (9, 5)], 0), Open(&[(0, 9), (1, 6), (3, 4)], 0)], 0),
];

const BUTTONS: &[Shp] = &[
    ("Action Button: Back or Previous", &[Bx(0, 0, 10, 10, 0), Poly(&[(6, 2), (2, 5), (6, 8)])], 0),
    ("Action Button: Forward or Next", &[Bx(0, 0, 10, 10, 0), Poly(&[(4, 2), (8, 5), (4, 8)])], 0),
    ("Action Button: Beginning", &[Bx(0, 0, 10, 10, 0), Poly(&[(7, 2), (3, 5), (7, 8)]), Open(&[(3, 2), (3, 8)], 0)], 0),
    ("Action Button: End", &[Bx(0, 0, 10, 10, 0), Poly(&[(3, 2), (7, 5), (3, 8)]), Open(&[(7, 2), (7, 8)], 0)], 0),
    ("Action Button: Home", &[Bx(0, 0, 10, 10, 0), Poly(&[(5, 2), (8, 5), (7, 5), (7, 8), (3, 8), (3, 5), (2, 5)])], 0),
    ("Action Button: Information", &[Bx(0, 0, 10, 10, 0), Ov(2, 2, 8, 8), Open(&[(5, 5), (5, 7)], 0), Open(&[(5, 3), (5, 4)], 0)], 0),
    ("Action Button: Return", &[Bx(0, 0, 10, 10, 0), Open(&[(7, 3), (7, 6), (3, 6)], 1)], 0),
    ("Action Button: Movie", &[Bx(0, 0, 10, 10, 0), Bx(2, 3, 6, 7, 0), Poly(&[(6, 5), (8, 3), (8, 7)])], 0),
    ("Action Button: Document", &[Bx(0, 0, 10, 10, 0), Poly(&[(3, 2), (6, 2), (7, 3), (7, 8), (3, 8)])], 0),
    ("Action Button: Sound", &[Bx(0, 0, 10, 10, 0), Poly(&[(2, 4), (4, 4), (6, 2), (6, 8), (4, 6), (2, 6)]), Open(&[(7, 3), (8, 5), (7, 7)], 0)], 0),
    (
        "Action Button: Help",
        &[Bx(0, 0, 10, 10, 0), Bez([(4, 3), (4, 1), (7, 1), (6, 4)], 0), Open(&[(5, 5), (5, 6)], 0), Open(&[(5, 7), (5, 8)], 0)],
        0,
    ),
    ("Action Button: Custom", &[Bx(0, 0, 10, 10, 0)], 0),
];

const CATEGORIES: [(&str, &[Shp]); 8] = [
    ("Lines", LINES),
    ("Rectangles", RECTANGLES),
    ("Basic Shapes", BASIC),
    ("Block Arrows", BLOCK_ARROWS),
    ("Equation Shapes", EQUATION),
    ("Flowchart", FLOWCHART),
    ("Stars and Banners", STARS),
    ("Callouts", CALLOUTS),
];

fn f(v: impl Into<f32>) -> f32 {
    v.into()
}

/// Draws one shape's primitives inside `r`.
fn paint(p: &Painter, r: Rect, xf: u8, prims: &[G], s: Stroke) {
    let pt = |x: f32, y: f32| {
        let (mut x, mut y) = (x / 10.0, y / 10.0);
        if xf & 4 != 0 {
            x = 1.0 - x;
        }
        for _ in 0..xf & 3 {
            (x, y) = (1.0 - y, x);
        }
        pos2(r.left() + x * r.width(), r.top() + y * r.height())
    };
    let ptu = |&(x, y): &(u8, u8)| pt(f(x), f(y));
    let draw = |pts: Vec<Pos2>, closed: bool| {
        p.add(if closed { Shape::closed_line(pts, s) } else { Shape::line(pts, s) });
    };
    let stroke_open = |pts: Vec<Pos2>, heads: u8| {
        let head = |tip: Pos2, from: Pos2| {
            let d = (tip - from).normalized() * 5.0;
            for a in [0.5, -0.5] {
                let (sin, cos) = f32::sin_cos(a);
                p.line_segment([tip, tip - vec2(d.x * cos - d.y * sin, d.x * sin + d.y * cos)], s);
            }
        };
        if let [.., a, b] = pts[..] {
            if heads & 1 != 0 {
                head(b, a);
            }
        }
        if let [a, b, ..] = pts[..] {
            if heads & 2 != 0 {
                head(a, b);
            }
        }
        draw(pts, false);
    };
    let polar = |n: u32, inner: f32| -> Vec<Pos2> {
        (0..n)
            .map(|i| {
                let a = i as f32 * TAU / n as f32 - FRAC_PI_2;
                let k = if i % 2 == 1 { inner } else { 1.0 };
                pt(5.0 + 5.0 * k * a.cos(), 5.0 + 5.0 * k * a.sin())
            })
            .collect()
    };
    for g in prims {
        match *g {
            Bx(x0, y0, x1, y1, rad) => {
                let b = Rect::from_two_pos(pt(f(x0), f(y0)), pt(f(x1), f(y1)));
                p.rect_stroke(b, f(rad) * r.width() / 10.0, s, StrokeKind::Middle);
            }
            Ov(x0, y0, x1, y1) => {
                let (cx, cy, rx, ry) = ((f(x0) + f(x1)) / 2.0, (f(y0) + f(y1)) / 2.0, (f(x1) - f(x0)) / 2.0, (f(y1) - f(y0)) / 2.0);
                draw((0..32).map(|i| pt(cx + rx * (i as f32 * TAU / 32.0).cos(), cy + ry * (i as f32 * TAU / 32.0).sin())).collect(), true);
            }
            Poly(pts) => draw(pts.iter().map(ptu).collect(), true),
            Open(pts, heads) => stroke_open(pts.iter().map(ptu).collect(), heads),
            Bez(c, heads) => {
                let [a, b, c, d] = c.map(|q| ptu(&q).to_vec2());
                let curve = (0..=16).map(|i| {
                    let (t, u) = (i as f32 / 16.0, 1.0 - i as f32 / 16.0);
                    (a * (u * u * u) + b * (3.0 * u * u * t) + c * (3.0 * u * t * t) + d * (t * t * t)).to_pos2()
                });
                stroke_open(curve.collect(), heads);
            }
            Arc(a0, a1, close) => {
                let mut v: Vec<Pos2> = (0..=24)
                    .map(|i| {
                        let a = (f(a0) + f(a1 - a0) * i as f32 / 24.0).to_radians();
                        pt(5.0 + 5.0 * a.cos(), 5.0 + 5.0 * a.sin())
                    })
                    .collect();
                if close == 1 {
                    v.push(pt(5.0, 5.0));
                }
                draw(v, close > 0);
            }
            Star(n, pct) => draw(polar(u32::from(n) * 2, f(pct) / 100.0), true),
            Reg(n) => draw(polar(u32::from(n), 1.0), true),
        }
    }
}

/// Excel's Shapes gallery: a "Recently Used" row (empty for now) then one tile grid per category.
/// The tiles are greyed out and do nothing yet.
pub(super) fn shapes_gallery(_app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let s = Stroke::new(1.2, t.text_dim.gamma_multiply(0.6));
    ui.set_width(268.0);
    egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
        ui.label(egui::RichText::new("Recently Used Shapes").strong());
        ui.label(egui::RichText::new("None yet").color(t.text_dim));
        for (name, shapes) in CATEGORIES {
            ui.label(egui::RichText::new(name).strong());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for &(tip, prims, xf) in shapes {
                    let (r, resp) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::hover());
                    paint(ui.painter(), r.shrink(5.0), xf, prims, s);
                    resp.on_hover_text(tip);
                }
            });
        }
    });
}
