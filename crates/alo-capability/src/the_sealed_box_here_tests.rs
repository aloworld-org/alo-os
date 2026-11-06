//! Reading whether `/` is a composefs image.

use super::the_root_is_composefs;

/// The root of an installed alo OS, as `findmnt` reads it: an overlay whose
/// source is composefs, over the deployment's `.ostree.cfs`.
const AN_INSTALLED_ALO_OS: &str = "\
22 1 0:23 / / ro,relatime shared:1 - overlay composefs ro,lowerdir+=/run/ostree/.private/cfsroot-lower,datadir+=/sysroot/ostree/repo/objects,redirect_dir=on,metacopy=on
23 22 0:24 / /sysroot ro,relatime shared:2 - btrfs /dev/vda3 rw,ssd,space_cache=v2,subvolid=5,subvol=/
40 22 0:24 /ostree/deploy/default/var /var rw,relatime shared:5 - btrfs /dev/vda3 rw,subvolid=5
";

/// An ordinary machine: `/` is a filesystem of its own.
const AN_ORDINARY_MACHINE: &str = "\
22 1 8:2 / / rw,relatime shared:1 - ext4 /dev/sda2 rw
30 22 0:5 / /dev rw,nosuid shared:2 - devtmpfs devtmpfs rw
";

#[test]
fn an_installed_alo_os_has_a_composefs_root() {
    assert!(the_root_is_composefs(AN_INSTALLED_ALO_OS));
}

#[test]
fn composefs_mounted_as_its_own_kind_is_read_too() {
    assert!(the_root_is_composefs(
        "22 1 0:23 / / ro shared:1 - composefs composefs ro\n"
    ));
}

/// **Only the root is asked about**: an overlay of composefs mounted
/// somewhere else — a container's, say — is not this machine's `/`.
#[test]
fn an_ordinary_root_is_not_and_a_composefs_elsewhere_does_not_count() {
    assert!(!the_root_is_composefs(AN_ORDINARY_MACHINE));
    assert!(!the_root_is_composefs(
        "22 1 8:2 / / rw shared:1 - ext4 /dev/sda2 rw\n\
         50 22 0:40 / /var/lib/containers/x ro - overlay composefs ro\n"
    ));
    assert!(!the_root_is_composefs(""));
    assert!(!the_root_is_composefs("not a mount line at all"));
}
