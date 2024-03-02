use lazy_static::lazy_static;
use mangui::cosmic_text::Metrics;
use mangui::femtovg::Color;

lazy_static!{
    pub static ref BACKGROUND: Color = Color::hex("282C34");
    pub static ref RED: Color = Color::hex("E06C75");
    pub static ref GREEN: Color = Color::hex("98C379");
    pub static ref YELLOW: Color = Color::hex("E5C07B");
    pub static ref BLUE: Color = Color::hex("61AFEF");
    pub static ref MAGENTA: Color = Color::hex("C678DD");
    pub static ref CYAN: Color = Color::hex("56B6C2");
    pub static ref GRAY: Color = Color::hex("ABB2BF");
    pub static ref WHITE: Color = Color::hex("FFFFFF");
    pub static ref BLACK: Color = Color::hex("000000");
}

pub static TEXT_NORMAL: Metrics = Metrics::new(16., 20.);
pub static TEXT_SMALL: Metrics = Metrics::new(14., 18.);
pub static TEXT_TINY: Metrics = Metrics::new(12., 16.);
pub static TEXT_LARGE: Metrics = Metrics::new(20., 24.);
pub static TEXT_HUGE: Metrics = Metrics::new(24., 30.);
pub static TEXT_GIANT: Metrics = Metrics::new(32., 36.);