# Prompt — validation P4 de la spec, Story 30-2 (des documents à traiter à la facture fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation
(D6, rétro 25) : P1 Sonnet → P2 Opus → P3 Sonnet → **P4 Opus** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`, tête `b1914712` (remédiation de la P3). Fiche :
`_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — une fiche, **trois** stories : 30-2a
(AC1-AC9, AC6-bis, AC9-bis), 30-2b (AC10-AC17, AC12-bis), 30-2c (AC18-AC25). Registre : `epic-30-choix-autonomes-302.md`
(C-30-2-1 à 23). Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § 30-2. Issue #685 —
**conception de Guy, elle fait foi** (`gh api repos/guycorbaz/kesh/issues/685` et `…/comments`, lecture seule). Rapports
précédents : `/home/gcorbaz/devel/kesh-gate-logs/302-validate-p{1,2,3}-{R,F}.md` ; leurs remédiations sont les entrées
« Validation P1 », « P2 » et « P3 » du Change Log. Règles : `CLAUDE.md` (Review Iteration Rule et ses sous-sections,
Règle de splitting préventif, Migration breaking policy P1-P8, Un appariement automatique propose, Issue Tracking Rule) ;
`docs/agents-orchestration.md`. La Story 30-1 (#686) sera développée avant : la fiche ne touche pas
`kesh-reconciliation`.

⚠️ **Motif mesuré sur cette fiche, trois fois** : la sévérité se déplace vers ce qu'on vient d'écrire. En P3, le seul
HIGH était une prémisse de la remédiation P2 (un « défaut d'origine » qui n'existait pas, pris pour acquis sans relire
`from_scanned`). Une affirmation sur le comportement du code ne vaut que par la ligne qui la prouve.

## Axe obligatoire (les deux lentilles) — relire la remédiation P3 ligne à ligne contre le code

Prends `git diff 1887aea6 b1914712 -- _bmad-output/implementation-artifacts/` et, **pour chaque ligne ajoutée** de la
fiche et du registre qui affirme un fait du dépôt (chemin, numéro de ligne, signature, comportement d'une fonction,
valeur de colonne, clé i18n, recette), vérifie-le au code par `grep -nF` ou `sed -n` et copie la preuve. Une affirmation
fausse ou invérifiable est au moins **MEDIUM** si un AC, un test ou une mutation repose dessus. À ne pas manquer :
`imported_supplier_invoice.rs:117-128` et `:256` ; `kesh-api/src/errors.rs:3281-3287` ; `parser.rs:145`, `:155-164` ;
`generator.rs:14-15` ; `inbox_import_e2e.rs:250-307` (helpers) ; `iban.rs:13-101`, `:135-136` ; `qr_iban.rs:38` ;
`validation.rs:23-30`, `:141` ; `crates/kesh-db/src/repositories/payment_batches.rs:359`, `:793` ;
`repositories/supplier_invoices.rs:271`, `:356` (sentinelle `companies`) ; `routes/journal_entries.rs:1007-1013` ;
`supplier-invoices/[id]/+page.svelte:393` ; `reports.api.ts:193` ; `messages.ftl:25`, `:2224` ;
`duplicate-probe.ts:64` ; `20260814000001_contacts_client_number_canonical.sql:6`. Le relevé au banc de l'orchestrateur
(AC19, C-30-2-21) est un fait mesuré hors dépôt : ne le conteste pas sans mesure ; vérifie en revanche que la fiche le
décrit de façon reproductible et que la bascule écrite est réalisable (ordre des verrous réécrit, cycles avec
`create_in_tx`, qui prend la même sentinelle).

## Axes prioritaires (communs aux deux lentilles)

1. **Cohérence d'ensemble après trois remédiations** : résidus des conceptions abandonnées (contrôle de domaine,
   `QRR_WITHOUT_QR_IBAN`, « forme QRR » refusée, 500 sur tout le run, `### Corrigé`, M-a13, huit motifs) ; AC, tâches,
   mutations, E2E, gardes et angles morts qui se contredisent ; entrées datées du Change Log annotées et non réécrites.
2. **Inventaire et parcours de #685** (points 1 à 7) : un AC testable et un E2E chacun ? Découpage et modules (cinq au
   plus par sous-story, la 30-2c gagnant AC25) ?
3. **Migration et version** (P1-P8) : AC4, AC9-bis, procédure de mutation à deux fichiers.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (l. 1570, 1712-1741, 801, 808) et
   `admin-manual.tex` (l. 793, 855-880, 1456, 1599, 2012-2025) ; ce que AC9, AC9-bis, AC17, AC23 et AC25 prescrivent
   d'écrire sera-t-il vrai au code prescrit (effet exact sur la création manuelle, pays absents, lot pain.001,
   propositions, détachement, #503) ? Contrôle aussi les **PDF**, aplatis vers la **sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`) — jamais vers un fichier.
5. **Recomptes** : 28 AC, 23 tâches, 38 mutations (M-a13 retirée) dont trois déclarées équivalentes (M-a14, M-b2, M-c7),
   neuf parcours E2E, 23 choix, le § *Gardes existantes*, chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC couvert par une tâche et un test **qui rougirait** ; chaque mutation tuée par le test nommé, chaque
  équivalence déclarée vraie ; AC12-bis : les sept motifs, leurs tests, leurs libellés ×4 ; AC14 : calcule à la main
  `supplierNameKey` sur les cas nommés avec la liste ordonnée des formes juridiques ; AC25 : `formatSwissDate` sur
  `paidAt` ; AC24 : la mesure de l'attente (`is_finished` à 300 ms) est-elle probante ; E2E : constantes d'octets (un PNG
  et un PDF minimaux écrits à la main sont-ils réalistes pour le décodeur et pour le navigateur ?), poignée de main,
  attribution des copies. Textes, i18n ×4, manuels, CHANGELOG et son bandeau, `api-external.md`, `README.md:44`. Écrans
  et rôles.
- **F — Faisabilité, sécurité, verrous et naissance de la vue** : la bascule d'AC19 (sentinelle `companies` en tête de
  transaction) forme-t-elle un cycle avec `supplier_invoices::create_in_tx` (même sentinelle, après le projet ?), avec la
  complétion, avec le détachement ? Le test d'AC24 et ses 300 ms : faux positif possible sous charge ? AC12-bis :
  normalisation des blancs Unicode avant `Iban::new` (contournement, double normalisation, divergence écran / serveur) ;
  `supplier-candidates` (bornage, coût, IBAN hors URL et journaux) ; contrôle pur (`normalize().scale()`, plage) ; filet
  `CheckConstraintViolation` sans test ; CSP finale et bornes d'AC6-bis ; lecture du document de l'écriture (garde
  paramétrée par module) et de la facture (exécuteur générique dans l'instantané).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. Dis pour chaque finding s'il vient de la **remédiation P3**, d'une remédiation antérieure ou de la conception d'origine. ⛔ **Écris le rapport d'un seul jet** : un seul fichier, écrit une fois à la fin, sans brouillon ni fichier intermédiaire. ⛔ **La
liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p4-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p4-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/`. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`,
**`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête
SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie standard** seulement, jamais vers
un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés : lecture, `grep`,
`sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext … -`.
