use crate::{
    QuantPerm,
    Heritage,
    TransitionHeritage,
    Dimension,
};

impl QuantPerm {

    /// Pure Geometric Return
    ///
    /// Exile returns a realized manifold from its
    /// current witness coordinate back to the
    /// canonical coordinate encoded by its PERM.
    ///
    /// No external projections.
    /// No mirror collapse.
    /// No ledger lookups.
    /// No memory dependencies.
    ///
    /// The return path is derived entirely from
    /// the manifold's current state.
    pub fn exile(
        mut self,
    ) -> Heritage {

        // 1. Preserve the currently observed coordinate
        let from = self.dimension();

        // 2. Restore the genesis geometry encoded by PERM
        self.set_initial_dimension_from_perm();

        // 3. Canonical origin coordinate
        let to = self.dimension();

        let mirror_scalar = to as u128;

        // 4. Expand the canonical coordinate into a
        //    deterministic 32-byte mirror representation
        let mut mirror_bytes = [0u8; 32];

        mirror_bytes[..16]
            .copy_from_slice(
                &mirror_scalar.to_le_bytes()
            );

        // 5. Compute return work
        let (
            tau,
            delta,
            gross_work,
        ) = Self::calculate_work(
            mirror_scalar,
            self.retained_mass,
            from,
            to,
        );

        // 6. Structural amortization
        let net_work =
        gross_work.saturating_sub(
        self.structural_value
        );
         self.structural_value =
         self.structural_value
         .saturating_add(
            gross_work
        );

        // 7. Commit state mutations
        self.retained_mass = tau;
        self.dimension = to;
        self.activation_count =
            self.activation_count
                .saturating_sub(1);

        // 8. Deterministic return receipt
        let transition =
            TransitionHeritage {
                tau,
                delta,
                gross_work,
                net_work,
                mirror_bytes,
            };

        Heritage {
            state: self,
            transition,
        }
    }
}
