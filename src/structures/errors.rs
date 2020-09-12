use std::fmt;

#[derive(Debug)]
pub enum RoyalError<'a> {
    PermissionError(PermissionType<'a>),
    MissingError(&'a str),
    SelfError(&'a str),
    UnsuccessfulError(&'a str)
}

impl fmt::Display for RoyalError <'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RoyalError::PermissionError(perm) => 
                write!(f, "{}", perm),
            RoyalError::MissingError(missing) => write!(f, "Please provide a {}!", missing),
            RoyalError::SelfError(cmd) => write!(f, "I don't think you can {} yourself.", cmd),
            RoyalError::UnsuccessfulError(cmd) => 
                write!(f, "{} unsuccessful. The user must be in the guild and the bot must be above the user's role!", cmd)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PermissionType<'b> {
    SelfPerm(&'b str),
    Mention(&'b str, &'b str)
}

impl fmt::Display for PermissionType <'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PermissionType::SelfPerm(perm) => write!(f, concat!("You can't execute this command because you're not a {} on this server! \n",
                "If you're a moderator, try configuring a new moderator role! Navigate to `help config` to see how to set the mod role for this server!"), perm),
            PermissionType::Mention(cmd, perm) => write!(f, "I can't {} an {}! Please demote the user and try again", cmd, perm)
        }
    }
}
