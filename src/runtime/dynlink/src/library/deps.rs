use tracing::{debug, error};

use crate::{DynlinkError, ECollector};

use super::{Library, LibraryLoader};

use elf::abi::DT_NEEDED;

impl Library {
    pub(crate) fn enumerate_needed(
        &self,
        loader: &mut impl LibraryLoader,
    ) -> Result<Vec<Library>, DynlinkError> {
        debug!("{}: enumerating dependencies", self);
        let elf = self.get_elf()?;
        let common = elf.find_common_data()?;

        Ok(common
            .dynamic
            .ok_or(DynlinkError::Unknown)?
            .iter()
            .filter_map(|d| match d.d_tag {
                DT_NEEDED => Some({
                    let name = common
                        .dynsyms_strs
                        .ok_or(DynlinkError::Unknown)
                        .map(|strs| {
                            strs.get(d.d_ptr() as usize)
                                .map_err(|_| DynlinkError::Unknown)
                        })
                        .flatten();
                    name.map(|name| {
                        let dep = loader.open(name.into());
                        if dep.is_err() {
                            error!("failed to resolve library {} (needed by {})", name, self);
                        }
                        dep.map(|dep| Library::new(dep, name.to_string()))
                    })
                    .flatten()
                }),
                _ => None,
            })
            .ecollect()?)
    }
}
