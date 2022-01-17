#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResampleMode {
    Linear,
    Sinc,
    None,
}
