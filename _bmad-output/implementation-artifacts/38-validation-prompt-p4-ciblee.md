# Prompt de validation P4 ciblée — fiches de l'Epic 38

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule lentille, braquée sur **la dernière remédiation**.
Dépôt `/home/beat/limu/compta/kesh`, branche `epic/38-paypal`. Fiche(s) indiquée(s) dans ta mission.

## Périmètre

- La remédiation à relire : `git diff f005ba2f a5682b2e -- <fiche>` (lecture seule) et l'entrée « validation P3 » du
  Change Log. Contexte : `epic-38-paypal.md` (O1–O12, sections « Validation P1/P2 », D-P2-1 à D-P2-5),
  `38-p2-inter-fiches.md`, fiches sœurs.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les fiches sœurs (codes, routes, champs camelCase, motifs de file, numéros d'AC de 38-6c1/38-6c2),
  comptablement juste, testée ? Contradiction, exigence irréalisable, renvoi faux ?
- **Points inter-fiches à appliquer s'ils visent ta fiche** :
  - **38-3a** : l'instantané `opening` = solde au **début effectif** du premier passage du compte
    (`max(fetch_from, history_start)`, 38-3b AC-7) ; point de contact de l'AC-6 : 38-6a lève `TxDocuments::Nothing`
    pour lire les factures d'un compte prestataire CHF (ventes et achats), même formulation que D-P2-2/D-P2-5.
  - **38-9a** : l'avertissement « ignorer un code qui change le solde → écart durable au contrôle de complétude » au
    champ des surcharges **et** au manuel ; le `as_of` rendu est l'instant du solde affiché ; les E2E demandent les
    réponses PayPal par défaut avec `paypalResponses: "default"` (38-3b AC-3).
- Hors périmètre : le reste des fiches. **Découpage de 38-9a** (proposé en P3) : refusé — 5 modules, à la limite sans la
  dépasser ; ne pas le reproposer sauf défaut nouveau.

## Interdits

N'écris que dans la (les) fiche(s) visée(s), par **Edit** seulement. Aucun script (ni ad hoc, ni heredoc, ni `sed`),
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-P4-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation P4 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés sans les
appliquer. Message final bref, un rapport par fiche.
