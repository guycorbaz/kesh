# Story 63-1 — prompt de la revue de code P6 (Opus, contexte frais ; ciblée, sauf si la remédiation de la P5 sort de son périmètre)

Tu conduis la **passe P6** de la story 63-1 de Kesh (P1 Opus, P2 Sonnet, P3 Haiku ciblée, P4 Opus, P5 Sonnet complète, **P6 Opus**). Contexte : logiciel de comptabilité suisse, tenu en production
par Guy ; la story corrige trois P1 (#1752, #1753, #1754) : les annulations depuis les fiches contre-passent à la date de l'écriture d'origine, non à celle du jour. Worktree `kesh-63-1`, branche
`story/63-1-annulations-datees-origine`. Rapport de la P5 : `$GATE_LOGS/e63/63-1-revue-p5.md` (à lire en premier). Tête de la P5 : `3c3ac4390` (+ le commit du prompt P6) ; **le commit de remédiation de la P5
est celui que l'orchestrateur te donne** : `git diff 3c3ac4390..<tête>`.

## Dès la sixième passe, l'orchestrateur ne remédie plus lui-même

Cette passe est la **sixième** (`CLAUDE.md`, § *Review Iteration Rule*). Si tu rends un finding au-dessus de LOW, **l'orchestrateur ne le corrige pas** : il le renvoie au développement (agent de
développement, contexte frais), puis une passe de plus se lance sur ce seul patch, dans la limite des 8 passes. Dis-le dans ton rapport, et écris pour chaque finding le correctif **à confier au
développement**, avec son périmètre exact (fichiers, lignes, assertion attendue). Un LOW de texte de la fiche reste corrigeable par toi à la clôture, si aucun finding n'est au-dessus de LOW.

## Ce que la P5 a trouvé (0/0/1/4 — tendance P1 0/0/1/13, P2 0/0/1/4, P3 0/0/1/4, P4 0/0/1/4, P5 0/0/1/4)

- **M1 (MEDIUM)** — `frontend/tests/e2e/annulations-datees-origine.spec.ts:235` affirme encore `toContainText('faites d\'abord verrouiller la période')` pour la confirmation d'annulation d'un
  **règlement** client ; le texte est devenu, à la remédiation de la P3 (`8f2699ceb`, clé `invoices-settlement-cancel-confirm`), « … la TVA non : un règlement n'en porte pas. Si cette période est
  verrouillée, l'annulation sera refusée : un administrateur doit d'abord faire reculer le verrou ». La spec est rouge en tête (test 1 sur 5, **les quatre suivants, série, ne tournent pas**) ;
  rejouée avec cette seule assertion corrigée, les cinq passent (dont l'assertion du solde en frais bancaires, `:294-297`, juste). `reconciliation-cancel.spec.ts` : 4/4 verts. L'inventaire
  (`inventaire-tva-annulation.md`, § 3.5) ne porte pas ce site : sa recherche par symptôme ne cherchait pas le **fragment de texte que les specs E2E affirment**.
- **L1** — dans les quatre `messages.ftl`, la remédiation a réécrit la ligne `invoices-write-off-cancelled` en `invoices-write-off-cancelled =<texte>` (espace après `=` supprimé : seules quatre
  lignes du dépôt ont cette forme) ; Fluent l'accepte, rien n'est rouge, mais le diff n'est pas voulu. À rétablir.
- **L2** — la raison donnée au § 5 (angle mort n° 5) de l'inventaire pour laisser « un règlement n'en porte pas » catégorique est fausse dans sa prémisse : « un cas que la saisie ne propose pas ».
  `SettleInvoiceDialog.svelte:159` propose tout compte actif et imputable hors débiteurs et devises, le compte d'impôt préalable compris. La conclusion se défend (usage aberrant ; le texte dit ce que
  Kesh écrit) mais la phrase de l'inventaire doit dire la vérité ; l'issue (refus du compte d'impôt préalable comme contrepartie, ou nuance du texte) reste à ouvrir par l'orchestrateur.
- **L3** — la fiche (Dev Agent Record, remédiation P4) dit la spec `reconciliation-cancel.spec.ts` « non rejouée » : rejouée par la P4 et la P5 (4/4) ; et la ligne `Status:` ne porte pas les passes P4 et P5.
- **L4** — un escompte ou une perte sur une facture sans part de TVA (taux 0, société non assujettie, reste de quelques centimes) montre « TVA comprise » : le texte est vrai à zéro près, l'écran ne sait pas
  la ventilation. Observation, non défaut ; issue `enhancement` possible.

## La remédiation de la P5 (développement, Opus 5.5)

- **Commits** : `e55a8df23` (code : spec E2E `:235` et espace de `invoices-write-off-cancelled` ×4) ; `6257440a4` (fiche). Le diff à relire est `git diff 126aabf7e..<tête>` (la remédiation
  seule), non plus `3c3ac4390..`.
- **Inventaire** (`$GATE_LOGS/e63/inventaire-tva-annulation.md`) : § 6 « Textes affirmés par les tests » (17 lignes, 40 assertions, 1 fausse corrigée) ; § 3.5 T9 ; § 5 angle mort n° 5 corrigé (L2). Refais
  l'extraction toi-même (point 3), ne te fie pas à ce tableau.
- **Gates du développement** (`$GATE_LOGS/e63/rem6/`) : les deux specs seules **9/9**, Vitest complet 5594, `check`, `lint-i18n-ownership`, `build`, `cargo test -p kesh-i18n` 42, `binary(textes_coherents)` 34/34.

## Ce que tu vérifies

1. **Le commit de remédiation de la P5**, seul : la spec E2E (assertion de `:235` : elle doit affirmer le texte neuf du règlement, **pas** une sous-chaîne trouvée par complaisance — lis la valeur au FTL fr-CH),
   les quatre `messages.ftl` (espace restauré, **aucune autre ligne changée** : `git diff` ligne à ligne), l'inventaire, la fiche.
2. **Rejoue la spec** `annulations-datees-origine.spec.ts` **entière** (cinq tests) et `reconciliation-cancel.spec.ts`, seules, contre un backend sur base jetable `kesh_e631_p6`, avec le montage de
   `docs/testing.md` § *Prérequis Playwright local* (`KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, SMTP factice, inbox/documents/backup sous `/tmp/claude-1000/`, `LANG=fr_CH.UTF-8`),
   frontend construit d'abord, port libre vérifié (`ss -ltn`), backend lancé par `( cd … && exec env … ) &` et arrêté par son PID réel (`ss -ltnp`). Pas d'E2E complet.
3. **La classe, par un autre chemin que celui de l'inventaire.** Les quatre passes précédentes ont cherché les textes **d'écran**, du manuel et des doc-comments. Cherche maintenant **tout test qui affirme un
   fragment de ces textes** : specs Playwright (`frontend/tests/e2e/`), tests Vitest (`*.test.ts`), tests Rust qui lisent un FTL (`textes_coherents.rs`, `crates/*/tests/`), captures de recette. Méthode :
   extrais les valeurs des clés `invoices-settlement-cancel-confirm`, `invoices-write-off-cancel-confirm`, `invoices-write-off-no-vat-cancel-confirm`, `supplier-invoices-cancel-confirm`,
   `supplier-invoices-settlement-cancel-confirm`, `supplier-invoices-unvalidate-confirm-body-cancelled`, `reconciliation-cancel-confirm-vat` (quatre locales) **et leurs états d'avant la story**
   (`git show a9b551d5a:<ftl>`), puis grep chaque **fragment de trois mots** de chaque valeur d'avant dans tout `frontend/` et `crates/` : tout test qui affirme un fragment disparu est un site. Une
   assertion dont les deux côtés sortent du même extracteur ne prouve rien : fais l'extraction toi-même.
4. **Les LOW de la P5** : dis lesquels la remédiation a traités et si le traitement est juste.
5. **Le manuel** (`docs/manual/`) contre le code, **PDF compris**, recompilé dans une **copie** sous `/tmp/claude-1000/` (PDF supprimés, `make fr/user-manual.pdf fr/admin-manual.pdf`), aplati
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, grepé **sans apostrophe**. La P5 l'a fait : rien à reprendre si le commit de remédiation ne touche pas `docs/manual/`.
6. **Rien d'autre n'a bougé** : `git diff --stat 3c3ac4390..<tête>` ne nomme que les sites ci-dessus ; aucun PDF commité.
7. **Vitest complet** si le commit touche un fichier du frontend sous `src/` ; sinon, seulement les fichiers de test touchés.

## Interdits

Tu n'écris aucun fichier du dépôt avant ta conclusion : pas d'`Edit`/`Write` sur le code, pas de `sed -i`, pas de `scripts/prepare-release.sh`, ni d'autre script qui écrit ; pas de `gh` en écriture ;
pas de `docker compose` ni de redémarrage du conteneur ; jamais `GRANT` ni `REVOKE` ; jamais `pkill`, `killall` ni `kill` par motif. Rien sous `~/kesh/`. Une copie temporaire d'une spec dans le dépôt est
tolérée pour un essai si tu la retires aussitôt et contrôles `git status --porcelain`. Gates lourds : `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock scripts/mem-guard.sh …`, base
`kesh_e631` ; Vitest et specs E2E sous `$GATE_LOGS/.gate-front.lock`.

## Rendu

Findings classés (CRITICAL, HIGH, MEDIUM, LOW) avec fichier:ligne, **prémisse lue au code**, correctif **à confier au développement**. Décompte et tendance ; verdict sur le point 3 (sites trouvés par le
nouveau chemin, ou preuve qu'il n'y en a pas) ; **axes exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas). Si aucun finding au-dessus de LOW : corrige en un lot les LOW de texte
de la fiche, une entrée finale au Change Log (tendance des six passes, modèles, signaux de recyclage), `Status:` réconcilié, boucle close.
