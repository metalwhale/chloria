use anyhow::Result;
use mockall::mock;

use private::{HashingAlgorithmBoxClone, Internal};

pub(crate) trait HashingAlgorithm: HashingAlgorithmBoxClone + Send + Sync + 'static {
    fn verify(&self, secret: &str, hashed_secret: &str) -> Result<bool>;
}

impl Clone for Box<dyn HashingAlgorithm> {
    fn clone(&self) -> Self {
        self.box_clone(Internal)
    }
}

mod private {
    use super::HashingAlgorithm;

    pub(crate) struct Internal;

    pub(crate) trait HashingAlgorithmBoxClone {
        // Sealed with an unused internal argument to prevent this method from being called directly outside
        fn box_clone(&self, _internal: Internal) -> Box<dyn HashingAlgorithm>;
    }

    impl<A> HashingAlgorithmBoxClone for A
    where
        A: HashingAlgorithm + Clone,
    {
        fn box_clone(&self, _internal: Internal) -> Box<dyn HashingAlgorithm> {
            Box::new(self.clone())
        }
    }
}

mock! {
    pub(in super::super) HashingAlgorithm {}

    impl HashingAlgorithm for HashingAlgorithm {
        fn verify(&self, secret: &str, hashed_secret: &str) -> Result<bool>;
    }

    impl HashingAlgorithmBoxClone for HashingAlgorithm {
        fn box_clone(&self, _internal: Internal) -> Box<dyn HashingAlgorithm>;
    }
}
