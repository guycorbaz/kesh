# Epic 51 — Réimputation

*Kickoff du 2026-11-21, par découpage de l'Epic 48 (décision du Project Lead du 2026-11-21).* Release visée : **après l'Epic 53**
(triage P1 du 2026-11-21 : les P1 passent devant) — l'epic n'entre dans `main` que terminé. Branche
`epic/51-reimputation`, partie de `main`, créée au lancement de sa première story. Migration : à établir par les fiches (aucune
annoncée).

## Objet

Étendre le geste « Changer le compte analytique » (facture client, publié en v0.19.0) à la facture fournisseur et à
l'écriture manuelle (#1167, #1449). Reprises de l'Epic 48 sous leur clé d'origine.

## Stories

| Story | Issues | État au kickoff | Détail |
|---|---|---|---|
| 48-1 — Réimputation d'une facture fournisseur | #1167 | fiche à reprendre (étape 0 : relire contre `main`) | `epic-48-corrections-0-20-0.md` § 48-1 |
| 48-2a — Réimputation d'une écriture manuelle, serveur | #1167, #1449 | fiche validée P1, remédiée | § 48-2 |
| 48-2b — Réimputation d'une écriture manuelle, écran | #1167, #1449 | fiche validée P1, remédiée | § 48-2 |
| 51-solde — Story de solde | plafond fixé au lancement | — | — |

#1167 se ferme avec la 48-2b (mot-clé de fermeture au corps de la PR seulement).

## Fichiers et modules

Ceux déclarés pour 48-1 et 48-2 (`epic-48-corrections-0-20-0.md`, § *Fichiers et modules touchés*). Recoupement connu :
`kesh-api/src/routes/journal_entries.rs` avec l'Epic 47 d'un contributeur.

## Dépendances

- Dépend de : aucune (la colonne `settlement_journal_entry_line_id` de l'Epic 43 est publiée).
- Bloque : aucune.
