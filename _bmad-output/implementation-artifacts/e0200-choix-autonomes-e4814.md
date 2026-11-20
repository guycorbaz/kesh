# Registre des choix autonomes — Story 48-14 (clé `e4814`)

Développement de la 48-14 (`48-14-solde-dette-epic.md`), Opus 5.5, 2026-11-19. Chaque choix : contexte, option retenue,
autres options, réversibilité.

## C-e4814-1 — La sonde et la reproduction de KF-063 sont commitées, derrière `#[ignore]`

- **Contexte** : la fiche (Tâches, Lot D, 1) laisse au développeur le choix entre une sonde non commitée et une sonde
  commitée derrière `#[ignore]`.
- **Retenu** : commitée. `Ecrivain::Sonde` relève chaque écriture (position, longueur, réécriture, cumul, fin) ;
  `Ecrivain::AncienCumul` garde l'ancien modèle (cumul des octets, réécritures comprises) ; le test
  `kf063_reproduction_deterministe` (ignoré, joué par `--ignored --nocapture`) établit la cause et la réponse de l'écrivain
  de D6, et vérifie la prémisse de la sonde (deux tours identiques jusqu'au manifeste exclu).
- **Autres options** : sonde jetable hors dépôt (la preuve d'AC13 ne se rejouerait plus) ; ancien écrivain retiré (la
  reproduction ne se rejouerait plus).
- **Réversible** : oui, en retirant le test ignoré et les deux variantes.

## C-e4814-2 — `wasPreviouslyRejected` vrai côté client, faux côté fournisseur

- **Contexte** : AC9 demande un `auto_match_rejected_at` non nul sur la transaction « d'un groupe » (l'autre gardant
  `NULL`), et `wasPreviouslyRejected` vrai pour l'un, faux pour l'autre ; le second groupe, lui, n'est pas dé-rapproché.
- **Retenu** : le groupe dé-rapproché du test client porte la date (vrai) ; celui du test fournisseur garde `NULL`
  (faux). Les deux valeurs de la clé sont ainsi constatées, chacune sur un bras. M5 (`auto_match_rejected_at = NULL` retiré
  de l'aide commune) rougit par le test client.
- **Autres options** : un troisième groupe dé-rapproché dans chaque test (montage plus lourd, même preuve).
- **Réversible** : oui.

## C-e4814-3 — Le relevé de la caractérisation vit dans `crates/kesh-db/tests/support/caracterisation.rs`

- **Contexte** : T8 et T9 relèvent le même état (transaction, écriture, audit) dans deux binaires de test.
- **Retenu** : un module de `tests/support/`, inclus par `#[path]` dans les deux binaires (patron de `jour.rs`), pour ne
  pas dupliquer le relevé (règle DRY).
- **Autres options** : le relevé copié dans chaque fichier.
- **Réversible** : oui.

## C-e4814-4 — Compiler hors verrou, jouer sous verrou : archives nextest

- **Contexte** : la voie base (`/tmp/station-heavy.lock` + `.gate.lock`) était tenue plusieurs heures d'affilée par
  d'autres travaux ; une demande `cargo nextest run` en file compile les sources **au moment où elle obtient le
  verrou**, ce qui interdisait de réécrire `grouped_settlements.rs` tant que la caractérisation sur l'ancien code
  n'avait pas tourné, et faisait compiler sous verrou.
- **Retenu** : `cargo nextest archive` (sous `mem-guard`, hors verrou) fige les binaires de test de l'arbre de travail
  — l'ancien code (contenu de `dbe67c669`), le code réécrit, chaque mutation Rust — ; un script unique
  (`jouer-archives.sh`) les joue sous une seule prise du verrou. *Corrigé à la revue P1 (A4)* : les journaux citent
  l'archive, **pas son commit** ; la provenance se déduit des horodatages (fiche, Dev Agent Record). Les archives
  pèsent plusieurs Go chacune, leurs extractions plusieurs dizaines : le 2026-11-20 elles ont rempli le disque et
  l'orchestrateur les a supprimées. Désormais : une archive à la fois, supprimée dès qu'elle a servi, ou le build
  ordinaire.
- **Autres options** : attendre le verrou avant chaque étape (plusieurs heures par étape) ; jouer hors verrou
  (interdit).
- **Réversible** : oui ; les archives sous `target/` partent avec le worktree.

## C-e4814-5 — Preuve du rouge de T6 et T7 par les rejeux de M2 et M1

- **Contexte** : la fiche demande un rouge « naturel » de T6 et T7 (code d'avant) et, en plus, les rejeux E2E de M2 et M1.
- **Retenu** : les rejeux de M2 (`if (!open) onSuccess();` remis, exactement le code d'avant) et de M1 (branche D1
  neutralisée, la chute d'avant) tiennent lieu du rouge naturel ; un build du code d'avant aurait demandé un troisième
  build et une prise de plus de la voie base.
- **Réversible** : oui (un rejeu sur le build de `0a7447001` peut s'ajouter à la clôture).
