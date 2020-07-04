use super::{parse_utils, ContentSource};

use std::io::Result;

// TO DO: make pub(crate)
pub fn tag_is_valid(s: &dyn ContentSource) -> Result<bool> {
    let mut r = s.open()?;
    let mut line = Vec::new();

    parse_utils::read_line(&mut r, &mut line)?;
    if let Some(object_id) = parse_utils::header(&line.as_slice(), b"object") {
        if !parse_utils::object_id_is_valid(&object_id) {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    parse_utils::read_line(&mut r, &mut line)?;
    if parse_utils::header(&line.as_slice(), b"type") == None {
        return Ok(false);
    }

    parse_utils::read_line(&mut r, &mut line)?;
    if parse_utils::header(&line.as_slice(), b"tag") == None {
        return Ok(false);
    }

    parse_utils::read_line(&mut r, &mut line)?;
    if let Some(_tagger) = parse_utils::header(&line.as_slice(), b"tagger") {
        Ok(parse_utils::attribution_is_valid(&line))
    } else {
        Ok(true)
        // tagger line does not need to be present
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag test-tag\n\
                  tagger A. U. Thor <tagger@localhost> 1 +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_object() {
        let cs = "".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object\tbe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "obejct be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object zz9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189 \n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9\n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_type() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type\tcommit\n\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  tpye commit\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_no_type() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag\tfoo\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tga foo\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tga foo\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn valid_no_tagger() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_tagger() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger \n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger a < 1 +000\n"
            .to_string();
        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger b <b@c> <b@c> 0 +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger A. U. Thor <foo 1 +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger A. U. Thor foo> 1 +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger 1 +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger a <b> +0000\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger a <b>\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger a <b> z\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);

        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag foo\n\
                  tagger a <b> 1 z\n"
            .to_string();

        assert_eq!(tag_is_valid(&cs).unwrap(), false);
    }
}
