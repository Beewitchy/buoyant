mod circle;
mod line;
mod no_shape;
mod rectangle;
mod rounded_rectangle;

pub use circle::Circle;
pub use line::Line;
pub use no_shape::NoShape;
pub use rectangle::Rectangle;
pub use rounded_rectangle::RoundedRectangle;

use crate::primitives::transform::CoordinateSpaceTransform;

use super::Point;

/// The element of a Bézier path.
///
/// A valid path has `MoveTo` at the beginning of each subpath.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathEl {
    /// Move directly to the point without drawing anything, starting a new
    /// subpath.
    MoveTo(Point),
    /// Draw a line from the current location to the point.
    LineTo(Point),
    /// Draw a quadratic bezier using the current location and the two points.
    QuadTo(Point, Point),
    /// Draw a cubic bezier using the current location and the three points.
    CurveTo(Point, Point, Point),
    /// Close off the path.
    ClosePath,
}

pub trait Shape: CoordinateSpaceTransform {
    type PathElementsIter<'iter>: Iterator<Item = PathEl> + 'iter
    where
        Self: 'iter;

    fn path_elements(&self, tolerance: u16) -> Self::PathElementsIter<'_>;

    /// The smallest rectangle that encloses the shape.
    fn bounding_box(&self) -> Rectangle;

    #[cfg(feature = "embedded-graphics")]
    type Draw<C: ::embedded_graphics::prelude::PixelColor>: self::embedded_graphics::DrawProvider<Self, C>;

    /// If the shape is a line, make it available.
    fn as_line(&self) -> Option<Line> {
        None
    }

    /// If the shape is a rectangle, make it available.
    fn as_rect(&self) -> Option<Rectangle> {
        None
    }

    /// If the shape is a rounded rectangle, make it available.
    fn as_rounded_rect(&self) -> Option<RoundedRectangle> {
        None
    }

    /// If the shape is a circle, make it available.
    fn as_circle(&self) -> Option<Circle> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapePathIter<const N: usize> {
    elements: [PathEl; N],
    index: usize,
}

impl<const N: usize> Iterator for ShapePathIter<N> {
    type Item = PathEl;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let element = self.elements[self.index];
            self.index += 1;
            Some(element)
        } else {
            None
        }
    }
}

impl<const N: usize> ShapePathIter<N> {
    #[must_use]
    pub const fn new(elements: [PathEl; N]) -> Self {
        Self { elements, index: 0 }
    }
}

/// Describes the relationship between two rectangles.
#[allow(dead_code, reason = "unused with some feature combinations")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Intersection {
    /// The other rectangle is completely inside this rectangle.
    Contains,
    /// The other rectangle partially overlaps with this rectangle.
    Overlaps,
    /// The other rectangle does not intersect with this rectangle.
    NonIntersecting,
}

#[cfg(feature = "embedded-graphics")]
pub mod embedded_graphics {
    use core::marker::PhantomData;

    use super::{PathEl, Point, Shape};
    use crate::primitives::transform::{CoordinateSpaceTransform, LinearTransform};

    use embedded_graphics::{
        Drawable,
        geometry::Point as EgPoint,
        prelude::{DrawTarget, PixelColor},
        primitives::{Line as EgLine, Primitive as EgPrimitive, PrimitiveStyle, StyledDrawable},
    };

    pub trait DrawProvider<Sh: Shape + ?Sized, C: PixelColor> {
        fn draw(
            target: &mut impl DrawTarget<Color = C>,
            shape: &Sh,
            transform: &LinearTransform,
            style: &PrimitiveStyle<C>,
        );
    }

    pub trait PrimitiveShape: Shape + CoordinateSpaceTransform {
        type Primitive<C: PixelColor>: EgPrimitive
            + StyledDrawable<PrimitiveStyle<C>, Color = C>;
    }

    #[derive(Debug)]
    pub struct PrimitiveDrawProvider<S: PrimitiveShape> {
        phantom: PhantomData<S>,
    }

    impl<Sh, C: PixelColor> DrawProvider<Sh, C> for PrimitiveDrawProvider<Sh>
    where
        Sh: PrimitiveShape + Into<Sh::Primitive<C>>,
    {
        fn draw(
            target: &mut impl DrawTarget<Color = C>,
            shape: &Sh,
            transform: &LinearTransform,
            style: &PrimitiveStyle<C>,
        ) {
            let primitive: Sh::Primitive<C> = shape.applying(transform).into();
            let styled = primitive.into_styled(*style);
            let _ = styled.draw(target);
        }
    }

    #[derive(Debug)]
    pub struct PathDrawProvider;

    impl<S: Shape, C: PixelColor> DrawProvider<S, C> for PathDrawProvider {
        fn draw(
            target: &mut impl DrawTarget<Color = C>,
            shape: &S,
            transform: &LinearTransform,
            style: &PrimitiveStyle<C>,
        ) {
            let offset = transform.offset;
            // Simplistic approach: convert each path segment to a line
            let mut last_point = None;

            for element in shape.path_elements(1) {
                match element {
                    PathEl::MoveTo(point) => {
                        last_point = Some(Point::new(point.x + offset.x, point.y + offset.y));
                    }
                    PathEl::LineTo(point) => {
                        if let Some(start) = last_point {
                            let end = Point::new(point.x + offset.x, point.y + offset.y);

                            let start_eg = EgPoint::new(start.x, start.y);
                            let end_eg = EgPoint::new(end.x, end.y);

                            let eg_line = EgLine::new(start_eg, end_eg).into_styled(*style);
                            let _ = eg_line.draw(target);

                            last_point = Some(end);
                        }
                    }
                    PathEl::QuadTo(_control, point) => {
                        // FIXME: Simplify quadratic curves to straight lines for now
                        if let Some(start) = last_point {
                            let end = Point::new(point.x + offset.x, point.y + offset.y);

                            let start_eg = EgPoint::new(start.x, start.y);
                            let end_eg = EgPoint::new(end.x, end.y);

                            let eg_line = EgLine::new(start_eg, end_eg).into_styled(*style);
                            let _ = eg_line.draw(target);

                            last_point = Some(end);
                        }
                    }
                    PathEl::CurveTo(_control1, _control2, point) => {
                        // FIXME: Simplify cubic curves to straight lines for now
                        if let Some(start) = last_point {
                            let end = Point::new(point.x + offset.x, point.y + offset.y);

                            let start_eg = EgPoint::new(start.x, start.y);
                            let end_eg = EgPoint::new(end.x, end.y);

                            let eg_line = EgLine::new(start_eg, end_eg).into_styled(*style);
                            let _ = eg_line.draw(target);

                            last_point = Some(end);
                        }
                    }
                    PathEl::ClosePath => {
                        // Close the path by drawing a line back to the starting point
                        if let (Some(start), Some(first)) = (last_point, last_point) {
                            let start_eg = EgPoint::new(start.x, start.y);
                            let end_eg = EgPoint::new(first.x, first.y);

                            let eg_line = EgLine::new(start_eg, end_eg).into_styled(*style);
                            let _ = eg_line.draw(target);
                        }
                    }
                }
            }
        }
    }
}
