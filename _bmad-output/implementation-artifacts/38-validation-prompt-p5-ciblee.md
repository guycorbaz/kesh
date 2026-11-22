# Prompt de validation P5 ciblée — fiches de l'Epic 38

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur **la dernière remédiation**.
Dépôt `<dépôt>`, branche `epic/38-paypal`. Fiche(s) indiquée(s) dans ta mission.

## Périmètre

- La remédiation à relire : `git diff 7e64e03b c6ea4402 -- <fiche>` (lecture seule) et l'entrée « validation P4 ciblée » du
  Change Log. Contexte : `epic-38-paypal.md` (O1–O12, sections « Validation P1/P2 », D-P2-1 à D-P2-5),
  `38-p2-inter-fiches.md`, fiches sœurs.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les fiches sœurs (codes, routes, champs camelCase, motifs de file, numéros d'AC de 38-6c1/38-6c2),
  comptablement juste, testée ? Contradiction, exigence irréalisable, renvoi faux ?
- **Points inter-fiches à appliquer s'ils visent ta fiche** :
  - **38-6a** AC-12 : la clé `reconciliation-review-reason-invoice-with-fee` (posée par 38-6a) porte ces textes :
    FR « Facture de Kesh avec frais : à régler à la main », DE « Rechnung in Kesh mit Gebühren: von Hand zu
    begleichen », IT « Fattura di Kesh con commissioni: da regolare a mano », EN « Kesh invoice with fees: settle by
    hand ».
  - **38-6b** AC-10 : la clé est posée par 38-6a AC-12 et reprise, jamais redéfinie.
  - **38-9b** : la clé est posée par 38-6a (non 38-6b) ; reformuler les deux endroits qui le disent autrement.
  - **38-7** AC-6 : un `opening` daté avant le début effectif est complété par 38-3b AC-7 (lecture de [A, D)).
  - **38-9a** : le motif `opening_unavailable` d'`errorDetail` (38-3b AC-7) a un libellé et un état « contrôle non
    disponible » à l'écran du contrôle, aux 4 locales.
- Hors périmètre : le reste des fiches. **Découpage de 38-9a** (proposé en P3) : refusé — 5 modules, à la limite sans la
  dépasser ; ne pas le reproposer sauf défaut nouveau.

## Interdits

N'écris que dans la (les) fiche(s) visée(s), par **Edit** seulement. Aucun script (ni ad hoc, ni heredoc, ni `sed`),
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-P5-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation P5 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés sans les
appliquer. Message final bref, un rapport par fiche.
