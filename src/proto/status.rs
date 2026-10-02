use std::fmt;

/// Standard binary HTTP response status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryStatus {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
}

impl BinaryStatus {
    pub fn code(self) -> u16 {
        self as u16
    }

    pub fn code_str(self) -> &'static str {
        match self {
            BinaryStatus::Ok => "200",
            BinaryStatus::BadRequest => "400",
            BinaryStatus::NotFound => "404",
            BinaryStatus::MethodNotAllowed => "405",
            BinaryStatus::InternalServerError => "500",
        }
    }

    pub fn reason_phrase(self) -> &'static str {
        match self {
            BinaryStatus::Ok => "OK",
            BinaryStatus::BadRequest => "Bad Request",
            BinaryStatus::NotFound => "Not Found",
            BinaryStatus::MethodNotAllowed => "Method Not Allowed",
            BinaryStatus::InternalServerError => "Internal Server Error",
        }
    }

    pub fn from_code(code: u16) -> Self {
        match code {
            200 => BinaryStatus::Ok,
            400 => BinaryStatus::BadRequest,
            404 => BinaryStatus::NotFound,
            405 => BinaryStatus::MethodNotAllowed,
            _ => BinaryStatus::InternalServerError,
        }
    }
}

impl fmt::Display for BinaryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason_phrase())
    }
}
