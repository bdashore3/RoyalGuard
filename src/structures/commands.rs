use serenity::framework::standard::macros::group;
use crate::commands::{
    general::*
};

// All command groups
#[group]
#[help_available(false)]
#[commands(ping)]
pub struct General;