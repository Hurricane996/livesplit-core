use {
    super::render,
    crate::{
        component,
        layout::{self, Component, Layout, LayoutDirection, LayoutState},
        run::parser::{livesplit, llanfair, wsplit},
        tests_helper, Run, Timer,
    },
    crc::crc32::checksum_ieee,
    image::Rgba,
    std::{
        fs::{self, File},
        io::BufReader,
    },
};

fn file(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn lss(path: &str) -> Run {
    livesplit::parse(file(path), None).unwrap()
}

fn lsl(path: &str) -> Layout {
    layout::parser::parse(file(path)).unwrap()
}

#[test]
fn default() {
    let mut run = tests_helper::create_run(&["A", "B", "C", "D"]);
    run.set_game_name("Some Game Name");
    run.set_category_name("Some Category Name");
    run.set_attempt_count(1337);
    let mut timer = Timer::new(run).unwrap();
    let mut layout = Layout::default_layout();

    tests_helper::start_run(&mut timer);
    tests_helper::make_progress_run_with_splits_opt(&mut timer, &[Some(5.0), None, Some(10.0)]);

    let state = layout.state(&timer);

    check(&state, 0x9df41392, "default");
}

#[test]
fn actual_split_file() {
    let run = lss("tests/run_files/livesplit1.0.lss");
    let timer = Timer::new(run).unwrap();
    let mut layout = Layout::default_layout();

    check(&layout.state(&timer), 0xb4f73995, "actual_split_file");
}

#[test]
fn wsplit() {
    let run = wsplit::parse(file("tests/run_files/wsplit"), false).unwrap();
    let timer = Timer::new(run).unwrap();
    let mut layout = lsl("tests/layout_files/WSplit.lsl");

    check_dims(&layout.state(&timer), [250, 300], 0x11c27904, "wsplit");
}

#[test]
fn all_components() {
    let mut layout = lsl("tests/layout_files/All.lsl");
    let run = lss("tests/run_files/livesplit1.6_gametime.lss");
    let mut timer = Timer::new(run).unwrap();
    tests_helper::start_run(&mut timer);
    tests_helper::make_progress_run_with_splits_opt(
        &mut timer,
        &[Some(10.0), None, Some(20.0), Some(55.0)],
    );

    let state = layout.state(&timer);

    check_dims(&state, [300, 800], 0x9dedc4e6, "all_components");

    check_dims(&state, [150, 800], 0x5bf536c8, "all_components_thin");
}

#[test]
fn score_split() {
    use crate::{component::timer, layout::ComponentState};

    let run = lss("tests/run_files/livesplit1.0.lss");
    let timer = Timer::new(run).unwrap();
    let mut layout = Layout::default_layout();

    let mut state = layout.state(&timer);
    let prev_seg = state.components.pop().unwrap();
    state.components.pop();
    let mut timer_state = timer::Component::new().state(&timer, layout.general_settings());
    timer_state.time = "50346".into();
    timer_state.fraction = "PTS".into();
    state.components.push(ComponentState::Timer(timer_state));
    state.components.push(prev_seg);

    check_dims(&state, [300, 400], 0xf110bfa0, "score_split");
}

#[test]
fn dark_layout() {
    let run = llanfair::parse(file("tests/run_files/llanfair")).unwrap();
    let timer = Timer::new(run).unwrap();
    let mut layout = lsl("tests/layout_files/dark.lsl");

    check(&layout.state(&timer), 0x9b5eddab, "dark_layout");
}

#[test]
fn subsplits_layout() {
    let run = lss("tests/run_files/Celeste - Any% (1.2.1.5).lss");
    let mut timer = Timer::new(run).unwrap();
    let mut layout = lsl("tests/layout_files/subsplits.lsl");

    tests_helper::start_run(&mut timer);
    tests_helper::make_progress_run_with_splits_opt(
        &mut timer,
        &[Some(10.0), None, Some(20.0), Some(55.0)],
    );

    check_dims(
        &layout.state(&timer),
        [300, 800],
        0x7a696b4f,
        "subsplits_layout",
    );
}

#[test]
fn display_two_rows() {
    let timer = tests_helper::create_timer(&["A"]);
    let mut layout = Layout::new();
    let mut component = component::text::Component::new();
    let settings = component.settings_mut();
    settings.display_two_rows = true;
    settings.text =
        component::text::Text::Split(String::from("World Record"), String::from("Some Guy"));
    layout.push(component);

    let mut component = component::delta::Component::new();
    component.settings_mut().display_two_rows = true;
    layout.push(component);

    check_dims(
        &layout.state(&timer),
        [200, 100],
        0x88688e34,
        "display_two_rows",
    );
}

#[test]
fn single_line_title() {
    let mut run = tests_helper::create_run(&["A"]);
    run.set_game_name("Some Game");
    run.set_category_name("Some Category");
    run.set_attempt_count(1337);
    let timer = Timer::new(run).unwrap();
    let mut layout = Layout::new();
    let mut component = component::title::Component::new();
    let settings = component.settings_mut();
    settings.display_as_single_line = true;
    settings.show_attempt_count = true;
    settings.show_finished_runs_count = true;
    layout.push(component);

    check_dims(
        &layout.state(&timer),
        [150, 30],
        0x9f92f324,
        "single_line_title",
    );
}

#[test]
fn horizontal() {
    let run = lss("tests/run_files/Celeste - Any% (1.2.1.5).lss");
    let mut timer = Timer::new(run).unwrap();
    let mut layout = Layout::default_layout();
    layout.general_settings_mut().direction = LayoutDirection::Horizontal;
    match &mut layout.components[1] {
        Component::Splits(splits) => splits.settings_mut().visual_split_count = 4,
        _ => unreachable!("We wanted to configure the splits"),
    }
    layout.push(component::separator::Component::new());
    layout.push(component::graph::Component::new());
    layout.push(component::separator::Component::new());
    layout.push(Box::new(
        component::detailed_timer::Component::with_settings(component::detailed_timer::Settings {
            display_icon: true,
            ..Default::default()
        }),
    ));

    tests_helper::start_run(&mut timer);
    tests_helper::make_progress_run_with_splits_opt(
        &mut timer,
        &[Some(10.0), None, Some(20.0), Some(55.0)],
    );

    check_dims(&layout.state(&timer), [1500, 40], 0x3a70ef32, "horizontal");
}

fn check(state: &LayoutState, expected_checksum: u32, name: &str) {
    check_dims(state, [300, 500], expected_checksum, name);
}

fn check_dims(state: &LayoutState, dims: [usize; 2], expected_checksum: u32, name: &str) {
    let image = render(state, dims);
    let calculated_checksum = checksum_ieee(&image);

    fs::create_dir_all("target/renders").ok();

    let path = format!("target/renders/{}_{:08x}.png", name, calculated_checksum);
    image.save(&path).ok();

    if calculated_checksum != expected_checksum {
        fs::create_dir_all("target/renders/diff").ok();

        let expected_path = format!("target/renders/{}_{:08x}.png", name, expected_checksum);
        if let Ok(expected_image) = image::open(expected_path) {
            let mut expected_image = expected_image.to_rgba();
            for (x, y, Rgba([r, g, b, a])) in expected_image.enumerate_pixels_mut() {
                if x < image.width() && y < image.height() {
                    let Rgba([r2, g2, b2, a2]) = image.get_pixel(x, y);
                    *r = (*r as i16).wrapping_sub(*r2 as i16).abs() as u8;
                    *g = (*g as i16).wrapping_sub(*g2 as i16).abs() as u8;
                    *b = (*b as i16).wrapping_sub(*b2 as i16).abs() as u8;
                    *a = (*a).max(*a2);
                }
            }
            let diff_path = format!("target/renders/diff/{}.png", name);
            expected_image.save(&diff_path).ok();
        }

        panic!(
            "Render mismatch for {} before: {:#08x}, after: {:#08x}",
            name, expected_checksum, calculated_checksum
        );
    }
}
