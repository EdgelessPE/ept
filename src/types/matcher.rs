use semver::VersionReq;

use super::verifiable::Verifiable;

#[derive(Clone, Debug, PartialEq)]
pub struct PackageMatcher {
    pub name: String,
    pub scope: Option<String>,
    pub mirror: Option<String>,
    pub version_req: Option<VersionReq>,
}

impl Verifiable for PackageMatcher {
    fn verify_self(&self, located: &String) -> anyhow::Result<()> {}
}
