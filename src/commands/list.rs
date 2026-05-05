use anyhow::{bail, Result};
use crate::{store::Store, tree};

pub fn run(store: &Store, subfolder: Option<&str>) -> Result<()> {
    if !store.root.exists() {
        bail!(
            "Password store not initialized at {}. Run: pass init <gpg-id>",
            store.root.display()
        );
    }
    tree::print_tree(&store.root, subfolder);
    Ok(())
}
