use crate::{QuantPerm,Heritage, TransitionHeritage, Dimension};

use crate::euclid::Euclid;
use crate::gravity::Gravity;
use crate::mirrorb::BiasMirror;

impl QuantPerm {

    pub fn exile( mut self, heritage: &Heritage, euclid: &Euclid) -> Heritage {

        //  1. Authenticate projection
    
        // 2. Current coordinate

        let from = self.dimension();

        // 3. Repulsive displacement

        let rev =
            BiasMirror::collapse(
                euclid,
                from as u128,
            );

        let to = rev.as_u128() as Dimension;

        // 4. Inherited gravity

        let gravity = heritage.transition.tau;
        

        // 5. Transport physics

        let ( payload, delta,net_work,) = Self::calculate_work(euclid, gravity, from, to, );

        // 6. Thermodynamic accumulation

        let gross_work =net_work.saturating_sub(self.structural_value
            );

        self.structural_value = self.structural_value.saturating_add(gross_work
                );

        // 7. Commit exile mutation

        self.retained_mass = payload;

        self.dimension = to;

        self.activation_count = self.activation_count
                .saturating_sub(1);

        // 8. Transition receipt

        let transition = TransitionHeritage {tau: payload, delta,
                gross_work,
                net_work,
                origin:
                    euclid.seed_type(),
            };

        let heritage = Heritage {state: self,
                transition,
            };

        // 9. Forensic projection

        Some(heritage)
    }

    pub fn calculate_work(
        euclid: &Euclid,
        gravity: u128,
        from: Dimension,
        to: Dimension,
    ) -> (u128, u128, u128) {

        // inverse curvature field

        let bias = BiasMirror::collapse(
                euclid,
                from as u128,
            );

        let scalar = bias.as_u128();

        // manifold displacement

        let diff = if to >= from {to - from} else {from - to};

        let delta = (diff as u128)
                .saturating_mul(180)
                .saturating_div(
                    u64::MAX as u128
                );

        // repulsive transport gravity

        let levity =
            Gravity::derive(
                gravity,
                scalar,
                delta,
            );

        let payload =
            levity.tau;

        // energetic displacement

        let net_work =
            payload.saturating_mul(
                delta
            );

        (
            payload,
            delta,
            net_work,
        )
    }
}
