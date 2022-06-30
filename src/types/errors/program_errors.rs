use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ProgramError {
    IO(io::Error),
    Parse(serde_json::Error),
    EntrypointNotFound(String),
}

impl From<serde_json::Error> for ProgramError {
    fn from(err: serde_json::Error) -> Self {
        ProgramError::Parse(err)
    }
}

impl From<io::Error> for ProgramError {
    fn from(err: io::Error) -> Self {
        ProgramError::IO(err)
    }
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::IO(error) => {
                write!(f, "IO error: ")?;
                error.fmt(f)
            }
            ProgramError::Parse(error) => {
                write!(f, "Parsing error: ")?;
                error.fmt(f)
            }
            ProgramError::EntrypointNotFound(entrypoint) => {
                f.write_fmt(format_args!("Entrypoint {} not found", entrypoint))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_entrypoint_not_found_error() {
        let error = ProgramError::EntrypointNotFound(String::from("my_function"));
        let formatted_error = format!("{}", error);
        assert_eq!(formatted_error, "Entrypoint my_function not found");
    }
}
