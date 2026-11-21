# Epic 50 — Corrections

*Kickoff du 2026-11-21, par découpage de l'Epic 48 (décision du Project Lead du 2026-11-21, revue des règles à plusieurs
développeurs : un epic tient dans une release, au plus six stories).* Release visée : **vendredi 2026-11-27**. Branche
`epic/50-corrections`, partie de `main` (`fe84ff9ce`, v0.19.1). Aucune migration.

## Objet

Deux corrections de défauts déjà en cours, reprises de l'Epic 48 avec leurs fiches, branches et worktrees, sous leur clé
d'origine (`48-11`, `48-12a`, `48-12b`).

## Stories

| Story | Issues | État au kickoff | Détail |
|---|---|---|---|
| 48-11 — Arrondi au centime | #1245, #1430 | développement en pause (travail non commité dans le worktree, à contrôler à la reprise) | `epic-48-corrections-0-20-0.md` § 48-11 |
| 48-12a — Doublons de factures fournisseur, serveur | #975 | revue de code P1 remédiée, P2 à jouer | `epic-48-corrections-0-20-0.md` § 48-12 |
| 48-12b — Doublons de factures fournisseur, écran | #985 | revue de code P1 remédiée, P2 à jouer | idem |
| 50-solde — Story de solde | plafond fixé à la reprise | — | dette des revues de l'epic seulement |

48-12a et 48-12b sortent ensemble, ou toutes deux à la release suivante.

## Fichiers et modules

Ceux déclarés au kickoff de l'Epic 48 pour 48-11, 48-12a et 48-12b (`epic-48-corrections-0-20-0.md`, § *Fichiers et modules
touchés*). Recoupement connu : `kesh-report/src/pdf.rs` et `csv.rs` (48-11) avec l'Epic 47 d'un contributeur ; celui-ci
arrive second et passe par la fonction commune d'arrondi.

## Dépendances

- Dépend de : aucune (la PR #1327 qu'attendait la 48-11 est publiée en 0.19.1).
- Bloque : aucune.

## Branches

Les branches `story/48-11-…` et `story/48-12-…` sont parties de `epic/48-corrections-0-20-0`, dont l'arbre égale `main`
depuis le 2026-11-21 (`fa9a4c4e0`) : elles se fusionnent dans `epic/50-corrections` sans reprise.
