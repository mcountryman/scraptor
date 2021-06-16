#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum FrameError {}

#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum DisplayError {}

#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum DriverError {}
