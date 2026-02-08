#![allow(clippy::match_same_arms)]

mod definitions;
mod hardware_input_input_line;
mod mock_data;
mod settings;
mod table;

use std::time::Instant;

use buoyant::{
    environment::DefaultEnvironment,
    event::{Event, EventContext, EventResult, Key, simulator::MouseTracker},
    focus::{DefaultFocus, FocusAction, FocusDirection, Role},
    primitives::Point,
    render::{AnimatedJoin, AnimationDomain, ContentShape, Render},
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget},
    view::prelude::*,
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use crate::{
    definitions::{GoodPixelColor, Page, PageAction, RenderData, State},
    mock_data::{PAGE_1, PAGE_2, SETTINGS},
};

const FONT: u8g2_fonts::FontRenderer =
    u8g2_fonts::FontRenderer::new::<u8g2_fonts::fonts::u8g2_font_t0_13_tf>();

pub fn view<'a, 'b, C: GoodPixelColor, F: Fn(&State) + 'a + Copy>(
    data: RenderData<'a, C>,
    state: &'b State,
    save_settings: F,
) -> impl View<C, State> + use<'a, C, F> {
    let paginate = move |s: &mut State, a: &buoyant::view::paginate::PageEvent| {
        s.page_action = Some(match *a {
            buoyant::view::paginate::PageEvent::Next => definitions::PageAction::Next,
            buoyant::view::paginate::PageEvent::Previous => definitions::PageAction::Prev,
            _ => return,
        });
        // Close any open inputs when changing pages
        s.opened_input = None;
        s.opened_cell_input = None;
        s.focused_table = false;
    };

    let state = state.clone();

    buoyant::view::Paginate::new(paginate, {
        let state = state.clone();
        buoyant::match_view!(data.page, {
            Page::IeTable {
                header,
                footer,
                names,
                ie,
                eu,
                table_dimensions: (r, c),
            } => VStack::new((
                hardware_input_input_line::hw_line(header, data.palette, false),
                table::table(data, &state, (r, c), names, ie, eu),
                hardware_input_input_line::hw_line(footer, data.palette, true),
            )),
            Page::Settings { header, footer } => VStack::new((
                hardware_input_input_line::hw_line(header, data.palette, false),
                settings::settings(data, &state, save_settings),
                hardware_input_input_line::hw_line(footer, data.palette, true),
            )),
        })
        .background_color(data.palette.dark_blue(), Rectangle)
    })
    .focus_touches()
    .map_event::<(), _>(|event: &Event, _state| match event {
        Event::KeyDown(key) => match key {
            Key::Character('h') | Key::LeftArrow => Some(FocusAction::Previous.into()),
            Key::Character('l') | Key::RightArrow => Some(FocusAction::Next.into()),
            Key::Character('k') | Key::UpArrow => Some(FocusAction::Previous.into()),
            Key::Character('j') | Key::DownArrow => Some(FocusAction::Next.into()),
            Key::Character('\n') => Some(FocusAction::Select.into()),
            Key::Character('e') | Key::Escape => Some(FocusAction::Blur.into()),
            _ => Some(event.clone()),
        },
        Event::KeyUp(_) => None,
        _ => Some(event.clone()),
    })
}

const PALETTE: definitions::Palette<Rgb888> = definitions::Palette::from_array([
    Rgb888::new(0x00, 0x00, 0x00),
    Rgb888::new(0x47, 0x47, 0xff),
    Rgb888::new(0x00, 0x00, 0x80),
    Rgb888::new(0x66, 0x66, 0x66),
    Rgb888::new(0x00, 0xbc, 0x10),
    Rgb888::new(0xd6, 0xd6, 0xd6),
    Rgb888::new(0xe3, 0x87, 0x0e),
    Rgb888::new(0xd1, 0x00, 0x00),
    Rgb888::new(0xff, 0xff, 0xff),
    Rgb888::new(0xe8, 0xf0, 0x00),
    Rgb888::new(0x9b, 0x30, 0xff),
]);

fn main() {
    let size = Size::new(320, 240);
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(size);
    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, PALETTE.black());
    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Coffeeeee", &output_settings);
    window.update(target.display()); // Prevent crash on start
    let app_start = Instant::now();
    let mut touch_tracker = MouseTracker::new();

    let mut app_state = definitions::State {
        static_ip: core::net::Ipv4Addr::new(192, 168, 11, 100),
        gateway: core::net::Ipv4Addr::new(192, 168, 11, 1),
        dns: core::net::Ipv4Addr::new(192, 168, 11, 137),
        dhcp: true,
        net_mask: 24,
        ..Default::default()
    };

    let mut page = mock_data::SETTINGS;

    let save_settings = |app_state: &definitions::State| {
        println!("Saving settings");
        println!("  IP: {}", app_state.static_ip);
        println!("  Gateway: {}", app_state.gateway);
        println!("  Net Mask: {}", app_state.net_mask);
        println!("  DNS: {}", app_state.dns);
        println!("  DHCP: {}", app_state.dhcp);
    };

    let mut root_view = {
        let render_data = RenderData {
            palette: &PALETTE,
            page,
        };
        view(render_data, &app_state, save_settings)
    };
    let mut view_state = root_view.build_state(&mut app_state);

    // Create initial source and target trees for animation
    let time = app_start.elapsed();
    let env = DefaultEnvironment::new(time);
    let layout = root_view.layout(&target.size().into(), &env, &mut app_state, &mut view_state);

    let mut source_tree = &mut root_view.render_tree(
        &layout.sublayouts,
        Point::default(),
        &env,
        &mut app_state,
        &mut view_state,
    );
    let mut target_tree = &mut root_view.render_tree(
        &layout.sublayouts,
        Point::default(),
        &env,
        &mut app_state,
        &mut view_state,
    );

    // obtain initial focus
    let mut focus_state = DefaultFocus::default_first();
    println!("{}", size_of_val(&focus_state));
    let result = root_view.handle_event(
        &Event::Focus(FocusAction::Focus(FocusDirection::Forward)),
        &EventContext::new(time).with_roles(Role::Button | Role::Container),
        target_tree,
        &mut app_state,
        &mut view_state,
        &mut focus_state,
    );
    let mut focus_rect = ContentShape::Empty;
    match result {
        EventResult::Handled { shape, .. } => {
            focus_rect = shape.clone();
        }
        EventResult::Deferred => (),
    }

    // Main event loop
    loop {
        let time = app_start.elapsed();
        let domain = AnimationDomain::top_level(time);
        let context = EventContext::new(time).with_roles(Role::Button | Role::Container);

        // Handle touch and keyboard events
        window
            .events()
            .filter_map(|event| {
                if event == embedded_graphics_simulator::SimulatorEvent::Quit {
                    std::process::exit(0);
                }
                touch_tracker.process_event(event)
            })
            .for_each(|event| {
                let result = root_view.handle_event(
                    &event,
                    &context,
                    target_tree,
                    &mut app_state,
                    &mut view_state,
                    &mut focus_state,
                );
                match result {
                    EventResult::Handled { shape, .. } => {
                        println!("{shape:?}");
                        focus_rect = shape.clone();
                    }
                    EventResult::Deferred => {
                        println!("Event deferred: {event:?}");
                        focus_rect = ContentShape::Empty;
                    }
                }
            });

        // Handle page changes
        if let Some((i, ie)) = app_state.ie_value_update.take() {
            println!("IE value update: {i}: {ie}");
        }
        if let Some(action) = app_state.page_action.take() {
            match (action, page) {
                (PageAction::Next, Page::IeTable { .. }) if page == PAGE_2 => page = SETTINGS,
                (PageAction::Next, Page::IeTable { .. }) => page = PAGE_2,
                (PageAction::Next, Page::Settings { .. }) => page = PAGE_1,

                (PageAction::Prev, Page::Settings { .. }) => page = PAGE_2,
                (PageAction::Prev, Page::IeTable { .. }) if page == PAGE_1 => page = SETTINGS,
                (PageAction::Prev, Page::IeTable { .. }) => page = PAGE_1,
            }
            // Rebuild view on page change
            context.request_view_rebuild();
        }

        // Only recompute the view, layout, and render trees if necessary
        if context.view_rebuild_requested.get() {
            // Join source and target trees at current time, "freezing" animation progress
            target_tree.join_from(source_tree, &domain);
            // Swap trees so the current target becomes the next source
            core::mem::swap(&mut source_tree, &mut target_tree);
            // Create new view and target tree
            let render_data = RenderData {
                palette: &PALETTE,
                page,
            };
            root_view = view(render_data, &app_state, save_settings);
            let env = DefaultEnvironment::new(time);
            let layout =
                root_view.layout(&target.size().into(), &env, &mut app_state, &mut view_state);
            *target_tree = root_view.render_tree(
                &layout.sublayouts,
                Point::default(),
                &env,
                &mut app_state,
                &mut view_state,
            );

            // Obtain updated focus + shape
            let result = root_view.handle_event(
                &Event::Focus(FocusAction::Focus(FocusDirection::Forward)),
                &context,
                target_tree,
                &mut app_state,
                &mut view_state,
                &mut focus_state,
            );
            match result {
                EventResult::Handled { shape, .. } => {
                    focus_rect = shape.clone();
                }
                EventResult::Deferred => {
                    println!("Focus not found")
                }
            }
        }

        // Only render if active animation was reported, the view changed, or redraw was requested
        if target.clear_animation_status()
            || context.view_rebuild_requested.get()
            || context.redraw_requested.get()
        {
            // Render animated transition between source and target trees
            Render::render_animated(
                &mut target,
                source_tree,
                target_tree,
                &PALETTE.white(),
                &domain,
            );
            // Draw focus overlay, if available
            use buoyant::primitives::transform::LinearTransform;
            use buoyant::render_target::{RenderTarget, SolidBrush, Stroke};
            let stroke = Stroke::new(2);
            let brush = SolidBrush::new(PALETTE.yellow());
            if std::env::var("DEBUG_FOCUS").is_ok() {
                match &focus_rect {
                    ContentShape::Rectangle(rect) => {
                        target.stroke(&stroke, LinearTransform::identity(), &brush, None, rect);
                    }
                    ContentShape::RoundedRectangle(rrect) => {
                        target.stroke(&stroke, LinearTransform::identity(), &brush, None, rrect);
                    }
                    ContentShape::Circle(circle) => {
                        target.stroke(&stroke, LinearTransform::identity(), &brush, None, circle);
                    }
                    ContentShape::Empty | _ => {}
                }
            }

            // Send to the display
            window.update(target.display());
            // Clear for the next frame
            target.clear(PALETTE.black());
        } else {
            // limit polling for updates to ~30 fps when idle
            std::thread::sleep(std::time::Duration::from_millis(33));
        }
    }
}
