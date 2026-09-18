use crate::{ControlKind, Error, Ui};

impl Ui {
    /// Checked values in HTML order, including disabled checked members.
    /// None means an unknown group; an existing group with nothing checked is empty.
    /// Duplicate member values are returned once for each checked member.
    pub fn checked_values(&self, group: &str) -> Option<Vec<&str>> {
        Some(
            self.checkbox_groups
                .get(group)?
                .iter()
                .filter_map(|&i| match &self.controls[i].kind {
                    ControlKind::Checkbox {
                        checked: true,
                        value,
                    } => Some(value.as_str()),
                    _ => None,
                })
                .collect(),
        )
    }

    /// True if any member was changed by the user this frame. Non-consuming;
    /// reset by end_frame. None means an unknown group.
    pub fn checkbox_group_changed(&self, group: &str) -> Option<bool> {
        Some(
            self.checkbox_groups
                .get(group)?
                .iter()
                .any(|&i| self.controls[i].changed),
        )
    }

    /// Atomically set the checked values; an empty slice clears the group.
    /// Unknown values/groups fail without changes. Duplicate values check every
    /// matching member. Host updates may change disabled members and do not emit
    /// user changed events, just like set_checked. Each checkbox's group and value are fixed when it is created.
    pub fn set_checked_values(&mut self, group: &str, values: &[&str]) -> Result<(), Error> {
        let members = self
            .checkbox_groups
            .get(group)
            .ok_or_else(|| Error::new("unknown checkbox group"))?;
        for value in values {
            if !members.iter().any(|&i| matches!(&self.controls[i].kind, ControlKind::Checkbox { value: v, .. } if v == value)) {
                return Err(Error::new(format!("unknown checkbox value: {value}")));
            }
        }
        let mut changed = false;
        for &i in members {
            if let ControlKind::Checkbox { checked, value } = &mut self.controls[i].kind {
                let next = values.contains(&value.as_str());
                changed |= *checked != next;
                *checked = next;
            }
        }
        if changed {
            self.invalidate_paint();
        }
        Ok(())
    }
}
