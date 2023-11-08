use crate::{config::MgmtConfig, Group};
use derive_more::Display;
use getset::CopyGetters;

#[derive(Debug, CopyGetters, Clone, PartialEq, Eq, Display)]
#[display(fmt = "{}", id)]
pub struct ResolvedGid {
    #[getset(get_copy = "pub")]
    gid: i32,
    #[getset(get_copy = "pub")]
    id: Group,
}

impl ResolvedGid {
    pub fn new(group: Group, config: &MgmtConfig) -> Self {
        let gid = match group {
            Group::Staff => config.staff_gid,
            Group::Student => config.student_gid,
            Group::Faculty => config.faculty_gid,
        };
        Self { gid, id: group }
    }
}

#[cfg(test)]
mod testing {
    use crate::{config::MgmtConfig, util::ResolvedGid, Group};

    #[test]
    fn take_gid_from_config() {
        let (staff_gid, student_gid, faculty_gid) = (42, 200, 3001);
        let config = MgmtConfig {
            staff_gid,
            student_gid,
            faculty_gid,
            ..Default::default()
        };

        assert_eq!(student_gid, ResolvedGid::new(Group::Student, &config).gid());
        assert_eq!(staff_gid, ResolvedGid::new(Group::Staff, &config).gid());
        assert_eq!(faculty_gid, ResolvedGid::new(Group::Faculty, &config).gid());
    }
}
