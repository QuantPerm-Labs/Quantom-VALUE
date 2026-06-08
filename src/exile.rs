use crate::{QuantPerm, Heritage, TransitionHeritage,Dimension,
};

use crate::euclid::Euclid;
use crate::gravity::Gravity;
use crate::mirrorb::BiasMirror;

impl QuantPerm {

    /// Pure State-Driven Thermodynamic Decay
    ///
    /// The decay path is determined
    /// solely by the current state variables 
    pub fn exile(
        mut self,
        euclid: &Euclid,
    ) -> Heritage {

        // 1. Current coordinate
        let from = self.dimension();

        // 2. Inverse projection from configuration space
        let inverse =
            BiasMirror::collapse(
                euclid,
                from as u128,
            );

        let to = inverse.as_u128() as Dimension;

        // 3. Fresh inverse physics derived purely from internal state mass
        let (tau, delta, gross_work,
        ) = Self::calculate_work(
            euclid,
            self.retained_mass,
            from,
            to,
        );

        // 4. Structural amortization against current structural value
        let net_work =
            gross_work.saturating_sub(
                self.structural_value(),
            );

        self.structural_value =
            self.structural_value
                .saturating_add(net_work);

        // 5. Commit state mutations to the current consumed instance
        self.retained_mass = tau;
        self.dimension = to;
        self.activation_count =
            self.activation_count
                .saturating_sub(1);

        // 6. Return a pristine deterministic receipt
        let transition =
            TransitionHeritage {
                tau,
                delta,
                gross_work,
                net_work,
                origin: euclid.seed_type(),
            };

        Heritage {
            state: self,
            transition,
        }
    }
}
