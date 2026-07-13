use iced::{
    Point, Size,
    border::Radius,
    widget::canvas::{Path, path::Builder},
};

pub fn standard_shape(variant: u8) -> Path {
    match variant {
        0 => Path::rounded_rectangle(Point::ORIGIN, Size::new(200., 100.), Radius::new(50.)),
        1 => {
            let mut diamond = Builder::new();
            diamond.move_to(Point::new(0.0, 50.0));
            diamond.line_to(Point::new(100.0, 0.0));
            diamond.line_to(Point::new(200.0, 50.0));
            diamond.line_to(Point::new(100.0, 100.0));
            diamond.close();
            diamond.build()
        }
        2 => {
            let mut squiggle = Builder::new();

            squiggle.move_to(Point::new(198.0, 11.0));
            squiggle.bezier_curve_to(
                Point::new(214.8, 54.8),
                Point::new(169.4, 102.6),
                Point::new(116.0, 89.0),
            );
            squiggle.bezier_curve_to(
                Point::new(94.6, 83.6),
                Point::new(74.4, 65.0),
                Point::new(44.0, 87.0),
            );
            squiggle.bezier_curve_to(
                Point::new(9.2, 112.2),
                Point::new(0.8, 97.6),
                Point::new(0.0, 61.0),
            );
            squiggle.bezier_curve_to(
                Point::new(-0.8, 25.0),
                Point::new(28.2, 0.4),
                Point::new(62.0, 5.0),
            );
            squiggle.bezier_curve_to(
                Point::new(108.4, 11.4),
                Point::new(113.8, 44.0),
                Point::new(168.0, 9.0),
            );
            squiggle.bezier_curve_to(
                Point::new(180.6, 1.0),
                Point::new(191.8, -5.2),
                Point::new(198.0, 11.0),
            );
            squiggle.build()
        }
        _ => panic!(),
    }
}
