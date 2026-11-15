# Prompt de validation P8 ciblée — fiches de l'Epic 38

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur **la dernière remédiation**.
Dépôt `/home/beat/limu/compta/kesh`, branche `epic/38-paypal`. Fiche(s) indiquée(s) dans ta mission.

## Périmètre

- La remédiation à relire : `git diff be2ce1a9 d3abf458 -- <fiche>` (lecture seule) et l'entrée « constats de la validation de l’Epic 40 » du
  Change Log. Contexte : `epic-38-paypal.md` (O1–O12, sections « Validation P1/P2 », D-P2-1 à D-P2-5),
  `38-p2-inter-fiches.md`, fiches sœurs.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les fiches sœurs (codes, routes, champs camelCase, motifs de file, numéros d'AC de 38-6c1/38-6c2),
  comptablement juste, testée ? Contradiction, exigence irréalisable, renvoi faux ?
- Constat à relire : la borne de `unreconciled` (AC-8 (b)) passe de `≤` à `<` `asOf`, alignée sur 38-3b AC-7 ; le cas sans `booked_at` garde une borne à la journée. Vérifie la concordance avec 38-3b AC-7 et 40-4.
- Hors périmètre : le reste des fiches. **Découpage de 38-9a** (proposé en P3) : refusé — 5 modules, à la limite sans la
  dépasser ; ne pas le reproposer sauf défaut nouveau.

## Interdits

N'écris que dans la (les) fiche(s) visée(s), par **Edit** seulement. Aucun script (ni ad hoc, ni heredoc, ni `sed`),
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-P8-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation P8 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés sans les
appliquer. Message final bref, un rapport par fiche.
