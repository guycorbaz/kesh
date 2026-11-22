# Prompt de validation P7 ciblée — fiches de l'Epic 38

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur **la dernière remédiation**.
Dépôt `<dépôt>`, branche `epic/38-paypal`. Fiche(s) indiquée(s) dans ta mission.

## Périmètre

- La remédiation à relire : `git diff d0b7c24f e7814a45 -- <fiche>` (lecture seule) et l'entrée « validation P6 ciblée » du
  Change Log. Contexte : `epic-38-paypal.md` (O1–O12, sections « Validation P1/P2 », D-P2-1 à D-P2-5),
  `38-p2-inter-fiches.md`, fiches sœurs.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les fiches sœurs (codes, routes, champs camelCase, motifs de file, numéros d'AC de 38-6c1/38-6c2),
  comptablement juste, testée ? Contradiction, exigence irréalisable, renvoi faux ?
- **Point inter-fiches à appliquer (38-9a seulement)** : 38-3b AC-7 (P5) écrit désormais dans `errorDetail` les motifs
  `opening_unavailable` et `opening_unavailable: span` (lecture de [A, D) non tentée au-delà de trois `max_window()`),
  et **joint plusieurs motifs par `; `** ; l'écran lit chaque motif séparément, chacun avec son libellé (4 locales) et
  l'état « contrôle non disponible » ; jamais le reste brut d'`errorDetail`.
- Hors périmètre : le reste des fiches. **Découpage de 38-9a** (proposé en P3) : refusé — 5 modules, à la limite sans la
  dépasser ; ne pas le reproposer sauf défaut nouveau.

## Interdits

N'écris que dans la (les) fiche(s) visée(s), par **Edit** seulement. Aucun script (ni ad hoc, ni heredoc, ni `sed`),
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-P7-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation P7 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés sans les
appliquer. Message final bref, un rapport par fiche.
