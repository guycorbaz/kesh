# Epic 31 — registre des choix autonomes de la Story 31-3 (Rust 1.97.1)

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-3

### C-31-3-1 — Une story courte, sans validation multi-passes

- **Contexte** : le bump touche une ligne de configuration, un `FROM`, cinq lignes de documentation et d'éventuels
  correctifs clippy mécaniques ; pas de logique, pas de schéma.
- **Retenu** : une seule passe P1 **ciblée**, une lentille F, Haiku, sur la fiche ; la revue de code se fera sur le diff
  réel (surtout les correctifs de lints).
- **Écartées** : deux lentilles Sonnet (R et F) comme pour une story fonctionnelle — coût sans objet ici.
- **Réversible** : oui.

### C-31-3-2 — Tag Docker `rust:1.97-bookworm` (mineur), comme `1.96` aujourd'hui

- **Contexte** : le `Dockerfile` fixe le tag mineur ; `rust-toolchain.toml` (copié dans l'image) fixe le patch.
- **Retenu** : garder la convention du tag mineur ; le dev vérifie l'existence du tag, sinon `rust:1.97.1-bookworm`.
- **Écartée** : épingler un digest — hors périmètre, cela change la politique de mise à jour de l'image.
- **Réversible** : oui.

### C-31-3-3 — Les toolchains 1.96 restent installées sur la station

- **Contexte** : des worktrees parallèles (31-1, 31-2) portent encore `channel = "1.96.1"` dans leur
  `rust-toolchain.toml` tant qu'ils n'ont pas rebasé sur `main` après ce merge.
- **Retenu** : le nettoyage (AC6) porte sur les **cibles cargo** de worktrees terminés, pas sur les toolchains.
- **Écartée** : `rustup toolchain uninstall 1.96.1` — casserait les branches qui n'ont pas rebasé.
- **Réversible** : oui (réinstallation).

### C-31-3-4 — Un lint nouveau se corrige ; l'exception se pose au site, jamais globalement

- **Contexte** : la règle de l'action A10 (« gate vert avec le nouveau clippy ») ne dit pas quoi faire d'un lint qui
  demanderait un changement de comportement.
- **Retenu** : correction mécanique par défaut ; sinon `#[allow]` local justifié et inscrit ici et au Dev Agent
  Record ; correction non mécanique = issue demandée à l'orchestrateur.
- **Écartée** : `-A clippy::x` en CI ou `#![allow]` de crate — éteint le lint pour tout le code futur.
- **Réversible** : oui.

### C-31-3-5 — Les mentions historiques de « Rust 1.96 » ne se réécrivent pas

- **Contexte** : `CHANGELOG.md` (v0.4.0) et le tableau de versions du README disent « Rust 1.96 » pour la v0.4.0.
- **Retenu** : laissées telles quelles — relevés datés (`CLAUDE.md` : un compte rendu daté ne se « met pas à jour »).
- **Réversible** : oui.

### C-31-3-6 — #697 intégrée à la 31-3 (choix de l'orchestrateur)

- **Contexte** : la fiche demandait d'ouvrir une issue pour l'absence de garde entre le `FROM` du `Dockerfile` et
  `rust-toolchain.toml` ; ouverte (#697). La règle de release fixe le plafond au tag v0.15.0 à moins de 136 issues
  ouvertes, et le compte est à 138 après trois issues de traçage nées des fiches.
- **Retenu** : intégrer #697 à la 31-3 — une garde dans `crates/kesh-api/tests/textes_coherents.rs` qui lit les deux
  valeurs et rougit si la version mineure diffère ; la PR de la 31-3 porte `closes #697`. C'est le site naturel : la 31-3
  bumpe justement les deux.
- **Écartée** : laisser #697 ouverte — la garde est petite et la 31-3 en est l'occasion.
- **Réversible** : oui.
