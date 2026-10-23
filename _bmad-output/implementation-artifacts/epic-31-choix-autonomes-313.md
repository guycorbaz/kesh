# Epic 31 — registre des choix autonomes de la Story 31-3 (Rust 1.97.1)

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-3

### C-31-3-1 — Une story courte, sans validation multi-passes

- **Contexte** : le bump touche une ligne de configuration, un `FROM`, sept sites de documentation (six lignes de la validation P1 plus `DOCKER_START.md`, validation P2 R8/F1) et d'éventuels
  correctifs clippy mécaniques ; pas de logique, pas de schéma.
- **Retenu** : une seule passe P1 **ciblée**, une lentille F, Haiku, sur la fiche (**révisé** : la fiche a changé de périmètre avec
  C-31-3-6, une P2 **complète** Sonnet, lentilles R et F, la suit) ; la revue de code se fera sur le diff
  réel (surtout les correctifs de lints).
- **Écartées** : deux lentilles Sonnet (R et F) comme pour une story fonctionnelle — coût sans objet ici.
- **Révisé le 2026-10-23 (validation P2, R5)** — titre et « Écartées » ci-dessus restent tels qu'à leur date ; ils ne
  valent plus. La fiche ayant intégré la garde G21 (C-31-3-6), la validation suit la règle commune (`CLAUDE.md` § *Review
  Iteration Rule*, `docs/agents-orchestration.md` § 3) : **passes complètes, Sonnet et Opus en alternance, lentilles R et F,
  jusqu'à une passe à 0 MEDIUM+ ou 8 passes**. La P2 (Opus) a rendu 6 MEDIUM + 1 MEDIUM ; la P3 (Sonnet) suit.
- **Réversible** : oui.

### C-31-3-2 — Tag Docker `rust:1.97-bookworm` (mineur), comme `1.96` aujourd'hui

- **Contexte** : le `Dockerfile` fixe le tag mineur ; `rust-toolchain.toml` (copié dans l'image) fixe le patch.
- **Retenu** : garder la convention du tag mineur ; le dev vérifie l'existence du tag, sinon `rust:1.97.1-bookworm`.
- **Écartée** : épingler un digest — hors périmètre, cela change la politique de mise à jour de l'image.
- **Réversible** : oui.

### C-31-3-3 — Les toolchains 1.96 restent installées sur la station

- **Contexte** : les **15 worktrees** de la station (dépôt principal, Epics 15, 29, 30 et 31, releases 0.13.4 et 0.14.0)
  portent tous `channel = "1.96.1"` dans leur `rust-toolchain.toml` (relevé du 2026-10-23, validation P1 F2), tant qu'ils
  n'ont pas rebasé sur `main` après ce merge.
- **Retenu** : le nettoyage (AC6) porte sur les **cibles cargo** de worktrees terminés, pas sur les toolchains.
- **Écartée** : `rustup toolchain uninstall 1.96.1` — casserait les branches qui n'ont pas rebasé.
- **Réversible** : oui (réinstallation).

### C-31-3-4 — Un lint nouveau se corrige ; l'exception se pose au site, jamais globalement

- **Contexte** : la règle de l'action A10 (« gate vert avec le nouveau clippy ») ne dit pas quoi faire d'un lint qui
  demanderait un changement de comportement.
- **Retenu** : correction mécanique par défaut ; sinon `#[allow]` local justifié et inscrit ici et au Dev Agent
  Record ; un lint non mécanique reçoit **les deux** : l'`allow` au site et une issue de suivi de son correctif, demandée à
  l'orchestrateur (une seule conduite, validation P1 F5).
- **Écartée** : `-A clippy::x` en CI ou `#![allow]` de crate — éteint le lint pour tout le code futur.
- **Réversible** : oui.

### C-31-3-5 — Les mentions historiques de « Rust 1.96 » ne se réécrivent pas

- **Contexte** : `CHANGELOG.md` (v0.4.0) et le tableau de versions du README disent « Rust 1.96 » pour la v0.4.0.
- **Retenu** : laissées telles quelles — relevés datés (`CLAUDE.md` : un compte rendu daté ne se « met pas à jour »).
- **Écartée** : réécrire ces mentions en 1.97 — cela ferait mentir la v0.4.0 sur la chaîne avec laquelle elle a été bâtie.
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
- **Justification (validation P2, R6 — arbitrage de l'orchestrateur)** : la règle du périmètre minimal
  (`docs/agents-orchestration.md` § 3 : un défaut voisin ne s'intègre que si sa correction est mécanique) **admet le budget
  d'issues comme motif d'intégration** — décision de Guy du 2026-10-23, `CLAUDE.md` § *Priorités des défauts*, « on trace
  toujours » : l'issue existe (#697), elle est tracée, et la fermer ici ramène le compte vers le plafond. La garde est
  petite et isolée (un test pur plus la lecture de deux fichiers, aucun code de production), et la 31-3 touche justement
  ces deux fichiers. Le corps de #697 disait « non mécanique » : c'est la fiche qui fait foi ; l'orchestrateur lèvera le
  « hors périmètre » de l'issue par un commentaire.
- **Annotation du 2026-10-23 (validation P3, R5)** : l'attribution ci-dessus est inexacte — `docs/agents-orchestration.md`
  § 3 ne pose que la règle du mécanique ; le budget d'issues comme motif **accepté** d'intégrer un défaut voisin est écrit
  au `CLAUDE.md` § *Priorités des défauts* (« ⛔ On trace toujours »), qui fait foi. La garde n'étant pas strictement
  mécanique, c'est ce second texte, et non le § 3, qui fonde l'intégration.

### C-31-3-7 — numéro non attribué

- Aucun choix ne porte ce numéro (trou de numérotation né des remédiations de la validation ; relevé par la validation P3,
  R4/F3). Il n'est **pas** réattribué et les suivants ne sont pas renumérotés : C-31-3-8 et C-31-3-9 sont cités par la
  fiche et par les rapports de passe.

### C-31-3-8 — G21 vit dans `textes_coherents.rs`, pas dans `configuration_transmise.rs`

- **Contexte** : `configuration_transmise.rs` lit déjà le `Dockerfile` (`lire("Dockerfile")` l. 1897, `dockerfile_fixe`
  l. 468-480) ; `textes_coherents.rs` garde des textes, dispose de `racine()`/`lire()`, de `regex` et de G19 comme modèle
  d'extraction adossée à une positive (validation P2, R14).
- **Retenu** : `textes_coherents.rs` — c'est l'emplacement que nomme #697, la garde compare deux *textes* de
  configuration, et la lecture qu'offre `configuration_transmise.rs` ne sert pas (`dockerfile_fixe` ne regarde que
  l'étape finale, après le dernier `FROM`, alors que G21 cherche l'étape `rust:`) : rien à mutualiser, la règle DRY n'est
  pas en jeu. G21 est libre (aucune occurrence dans les 15 arbres de travail).
- **Écartée** : `configuration_transmise.rs` — ses gardes G14-G17 portent sur l'environnement transmis à l'image, pas sur
  sa chaîne de build.
- **Réversible** : oui.

### C-31-3-9 — `rust-toolchain.toml` d'abord, puis les correctifs de lints

- **Contexte** : la première rédaction relevait les lints sous `+1.97.1`, corrigeait, puis bumpait. Sous 1.96.1, le gate
  ciblé ne voit pas les lints corrigés, et un correctif employant une API stabilisée en 1.97 (cas courant des lints
  `manual_*`, aucun `rust-version`/`msrv` ne le bornant) ne compile pas (validation P2, R3).
- **Retenu** : T2 bumpe `rust-toolchain.toml` seul, avant les correctifs ; chaque commit compile sous la chaîne qu'il porte ;
  le gate ciblé de T3 tourne sous 1.97.1. Le désaccord transitoire avec le `Dockerfile` est sans conséquence (G21 arrive à
  T4, et son test du dépôt est alors réellement rouge avant le bump du `Dockerfile`).
- **Écartée** : corriger sous `+1.97.1` en laissant 1.96.1 dans le fichier, en exigeant que chaque correctif compile aussi
  sous 1.96.1 — contrainte inutile et invérifiable commit par commit.
- **Réversible** : oui.

### C-31-3-10 — Sous-section `### Modifié`, et non `### Changed`, au CHANGELOG

- **Contexte** : T8 prescrit d'ajouter la ligne de la chaîne « sous `### Changed` ». Le `CHANGELOG.md` nomme ses
  sous-sections en français depuis la v0.4.0 (`### Ajouté`, `### Modifié`, `### Corrigé` : douze `### Modifié`) ; les trois
  `### Changed` restants appartiennent à des versions antérieures (l. 852, 870, 901). `scripts/prepare-release.sh` ne lit
  aucune sous-section (`grep` : aucune occurrence de `Changed` ni de `Modifié`).
- **Retenu** : `### Modifié`, convention vivante du fichier ; c'est l'intention de la fiche (la section des changements),
  écrite dans la langue du fichier.
- **Écartée** : `### Changed` à la lettre — une section 0.15.0 seule à mélanger les deux langues ; une autre story qui
  ajouterait `### Modifié` à la même section créerait deux sous-sections homologues au rebase.
- **Réversible** : oui.
