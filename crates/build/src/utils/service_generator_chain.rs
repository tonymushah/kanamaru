use std::ops::{Deref, DerefMut};

use prost_build::ServiceGenerator;

type Inner = Vec<Box<dyn ServiceGenerator>>;

/// A very simple and strait-forward ServiceGeneratorChain
pub struct ServiceGeneratorChain(Inner);

impl Deref for ServiceGeneratorChain {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServiceGeneratorChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ServiceGenerator for ServiceGeneratorChain {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        for generator in self.iter_mut() {
            generator.generate(service.clone(), buf);
        }
    }
    fn finalize(&mut self, _buf: &mut String) {
        for generator in self.iter_mut() {
            generator.finalize(_buf);
        }
    }
    fn finalize_package(&mut self, _package: &str, _buf: &mut String) {
        for generator in self.iter_mut() {
            generator.finalize_package(_package, _buf);
        }
    }
}
