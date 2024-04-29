use std::time::SystemTime;

use diligent_date_parser::parse_date;
use udf::prelude::*;
use ulid::Ulid as UlidGenerator;

#[derive(PartialEq, Debug)]
struct Ulid(UlidGenerator);

#[register]
impl BasicUdf for Ulid {
    type Returns<'a> = String;

    fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        cfg.set_max_len(26);
        cfg.set_maybe_null(false);
        cfg.set_is_const(false);
        return match args.len() {
            0 => Ok(Self(UlidGenerator::new())),
            1 => Self::parse_args(args),
            _ => Err(format!("expected 0 or 1 argument; got {}", args.len())),
        };
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        Ok(self.0.to_string())
    }
}

impl Ulid {
    fn parse_args(args: &ArgList<Init>) -> Result<Self, String> {
        return match args.get(0) {
            None => Ok(Self(UlidGenerator::new())),
            Some(arg) => match arg.value().as_string() {
                None => Ok(Self(UlidGenerator::new())),
                Some(date) => match parse_date(date) {
                    None => Err("unable to parse date format".to_string()),
                    Some(date) => Ok(Self(UlidGenerator::from_datetime(SystemTime::from(date)))),
                },
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use udf::chrono::{DateTime, NaiveDateTime, Utc};
    use udf::mock::*;
    use ulid::Ulid as UlidGenerator;

    // Use our parent module
    use super::*;

    #[test]
    fn test_default() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![];
        let mut rt = Ulid::init(cfg.as_init(), args.as_init()).unwrap();
        let res = rt.process(cfg.as_process(), args.as_process(), None);
        assert!(res.is_ok());
        let expected_datetime: DateTime<Utc> = UlidGenerator::new().datetime().into();
        let result_datetime: DateTime<Utc> = UlidGenerator::from_string(res.unwrap().as_str())
            .unwrap()
            .datetime()
            .clone()
            .into();
        assert_eq!(expected_datetime.timestamp(), result_datetime.timestamp())
    }

    #[test]
    fn test_null() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "NULL", true)];
        let mut rt = Ulid::init(cfg.as_init(), args.as_init()).unwrap();
        let res = rt.process(cfg.as_process(), args.as_process(), None);
        assert!(res.is_ok());
        let expected_datetime: DateTime<Utc> = UlidGenerator::new().datetime().into();
        let result_datetime: DateTime<Utc> = UlidGenerator::from_string(res.unwrap().as_str())
            .unwrap()
            .datetime()
            .clone()
            .into();
        assert_eq!(expected_datetime.timestamp(), result_datetime.timestamp())
    }

    #[test]
    fn test_from_datetime() {
        let mut cfg = MockUdfCfg::new();
        let date = "1983-04-13 12:09:14.274";
        let mut args = mock_args![(date, "", true)];
        let mut rt = Ulid::init(cfg.as_init(), args.as_init()).unwrap();
        let res = rt.process(cfg.as_process(), args.as_process(), None);
        assert!(res.is_ok());
        let expected_datetime = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap()
            .and_utc();
        let result_datetime: DateTime<Utc> = UlidGenerator::from_string(res.unwrap().as_str())
            .unwrap()
            .datetime()
            .clone()
            .into();
        assert_eq!(result_datetime.timestamp(), expected_datetime.timestamp())
    }

    #[test]
    fn test_wrong_args() {
        let mut cfg = MockUdfCfg::new();
        let mut arglist = mock_args![(String None, "", true), (String None, "", true)];
        let res = Ulid::init(cfg.as_init(), arglist.as_init());

        assert_eq!(res, Err("expected 0 or 1 argument; got 2".to_owned()));
    }
}
