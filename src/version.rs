#[derive(clap::ValueEnum, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Version {
    #[value(name = "closed_alpha")]
    ClosedAlpha,
    #[value(name = "closed_beta")]
    ClosedBeta,
}
