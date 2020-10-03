use super::{parse_utils, ContentSource, ContentSourceResult};

pub(crate) fn commit_is_valid(s: &dyn ContentSource) -> ContentSourceResult<bool> {
    let mut r = s.open()?;

    if let Some(line) = parse_utils::read_line(&mut r)? {
        if let Some(tree_id) = parse_utils::header(&line.as_slice(), b"tree") {
            if !parse_utils::object_id_is_valid(&tree_id) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    let line = loop {
        if let Some(line) = parse_utils::read_line(&mut r)? {
            if let Some(parent_id) = parse_utils::header(&line.as_slice(), b"parent") {
                if !parse_utils::object_id_is_valid(&parent_id) {
                    return Ok(false);
                }
            } else {
                break line;
            }
        } else {
            return Ok(false);
        }
    };

    if let Some(_author) = parse_utils::header(&line.as_slice(), b"author") {
        if !parse_utils::attribution_is_valid(&line) {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    if let Some(line) = parse_utils::read_line(&mut r)? {
        if let Some(_committer) = parse_utils::header(&line.as_slice(), b"committer") {
            if !parse_utils::attribution_is_valid(&line) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_no_parent() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn valid_blank_author() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                committer <> 0 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_corrupt_attribution() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                  committer b <b@c> <b@c> 0 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn valid_parents() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), true);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn valid_normal_time() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1222757360 -0730\n\
                  committer A. U. Thor <author@localhost> 1222757360 -0730\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_tree() {
        let cs = "parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "trie be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "treebe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree\tbe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree zzzzfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189z\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9b\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree  be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_parent() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent \n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent zzzzfa841874ccc9f2ef7c48d0c76226f89b7189\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189 \n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent  be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent be9bfa841874ccc9f2ef7c48d0c76226f89b7189z\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  parent\tbe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_no_author() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_author() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author b <b@c> <b@c> 0 +0000\n\
                  committer <> 0 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <foo 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor foo> 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author 1 +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author a <b> +0000\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author a <b>\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author a <b> z\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author a <b> 1 z\n"
            .to_string();

        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_no_committer() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);

        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\n"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_committer() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author a <b> 1 +0000\n\
                  committer a <"
            .to_string();
        assert_eq!(commit_is_valid(&cs).unwrap(), false);
    }
}
