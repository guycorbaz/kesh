# Prompt — validation P3 de la spec, Story 30-2 (des documents à traiter à la facture fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation
(D6, rétro 25) : P1 Sonnet → P2 Opus → **P3 Sonnet** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`, tête `e88cbc02` (remédiation de la P2 en `b00ee4b7`).
Fiche : `_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — une fiche, **trois** stories :
30-2a (AC1-AC9, AC6-bis, AC9-bis), 30-2b (AC10-AC17, AC12-bis), 30-2c (AC18-AC24). Registre de la story :
`epic-30-choix-autonomes-302.md` (C-30-2-1 à 22). Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md`
§ 30-2. Issue #685 — **conception de Guy, elle fait foi** (`gh api repos/guycorbaz/kesh/issues/685` et `…/comments`,
lecture seule). Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/302-validate-p1-{R,F}.md` et
`302-validate-p2-{R,F}.md` ; leurs remédiations sont les entrées « Validation P1 » et « Validation P2 » du Change Log.
Règles : `CLAUDE.md` (Review Iteration Rule et ses sous-sections, Règle de splitting préventif, Migration breaking
policy P1-P8, Un appariement automatique propose, Issue Tracking Rule) ; `docs/agents-orchestration.md`. La Story 30-1
(#686) sera développée avant : la fiche ne touche pas `kesh-reconciliation`.

⚠️ **Motif mesuré sur cette fiche** : en P2, douze MEDIUM sur treize venaient de la remédiation P1 — des prémisses
écrites contre la fiche et non contre le code (« le montant est borné par le parseur », un nom de fichier de plus de
255 caractères, `validate_iban` présenté comme un contrôle de somme). La remédiation P2 a réécrit AC2, AC12, **AC12-bis**,
AC14, AC6-bis, AC9, AC9-bis, AC16, AC17, AC19, AC21, AC24, le § E2E, les mutations et les tâches.

## Axe obligatoire (les deux lentilles) — relire la remédiation P2 ligne à ligne contre le code

Prends `git diff 58f65937 e88cbc02 -- _bmad-output/implementation-artifacts/` et, **pour chaque ligne ajoutée** de la
fiche et du registre qui affirme un fait du dépôt (un chemin, un numéro de ligne, une signature, un comportement d'une
fonction, une valeur de colonne, une recette, une clé i18n), vérifie-le au code par `grep -nF` ou `sed -n` et copie la
preuve. Une affirmation fausse ou non vérifiable est au moins **MEDIUM** si un AC, un test ou une mutation repose dessus.
Points à ne pas manquer : `kesh_core::types::Iban::new` et `QrIban::new` (ce qu'ils acceptent réellement : espaces,
casse, longueur par pays, CH/LI) ; `kesh_qrbill::validation::{amount_min, amount_max, validate_qrr}` ; `parser.rs:120`,
`:137`, `:145`, `:150` ; `inbox_import.rs:54`, `:515`, `:550-573` ; `errors.rs` (4025, 1264) ; la recette E2E de
`docs/testing.md` (l. 165-181) et ses répertoires ; `routes/journal_entries.rs:479`, `:983-1036`, `:1007` ;
`routes/supplier_invoices.rs:398-401` ; `bank-accounts/+page.svelte:276-281` ; `messages.ftl:2168-2172`, `:2223`,
`:2536` ; `closing-checks-labels.ts:63` ; `open-items.spec.ts:181` ; `payment-batches.spec.ts:107-108` ; la collation et le
mode de recherche des contacts (`repositories/contacts.rs:177` et suivantes, `util/search.rs:87`) — la variante
`muller` trouve-t-elle « Müller » et « Mueller » ?

## Axes prioritaires (communs aux deux lentilles)

1. **Cohérence d'ensemble après deux remédiations** : résidus de conceptions abandonnées (lecture verrouillante de
   l'étape 3, `validate_iban`, `isQrIban`, longueurs seules, éligibilité « tous les mots du plus court », `test.skip`,
   E6 dépendant d'E1, nom de fichier trop long) ; AC, tâches, mutations, E2E et gardes qui se contredisent.
2. **Inventaire et parcours de #685** (points 1 à 7) : chacun a-t-il toujours un AC testable et un E2E ? Découpage et
   modules par sous-story tiennent-ils encore (la 30-2b gagne une route `supplier-candidates`) ?
3. **Migration et version** (P1-P8) : AC4, AC9-bis ; mutations portées sur migration **et** squash (M-a5, M-c2) — la
   procédure de mutation de la fiche (motif unique, `git checkout -- f && touch f`) suffit-elle pour deux fichiers ?
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (l. 1570, 1712-1741, 801, 808) et
   `admin-manual.tex` (l. 793, 855-880, 1456, 1599, 2012-2025) ; ce que AC9, AC9-bis, AC17 et AC23 prescrivent d'écrire
   sera-t-il vrai au code prescrit (validation des IBAN telle qu'elle est réellement, propositions, détachement, #503) ?
   Contrôle aussi les **PDF**, aplatis vers la **sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`) — jamais vers un fichier.
5. **Recomptes** : 27 AC, 23 tâches, 38 mutations dont trois déclarées équivalentes (M-a14, M-b2, M-c7), neuf parcours
   E2E, 22 choix, le § *Gardes existantes*, chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC couvert par une tâche et un test **qui rougirait** ; chaque mutation tuée par le test nommé, chaque
  équivalence déclarée vraie (M-a14 : le filet `CheckConstraintViolation` est-il réellement inatteignable si le contrôle
  de domaine est juste — y a-t-il une autre contrainte CHECK sur le chemin, dont la « tout ou rien » d'AC4 ?) ; AC14 :
  `supplierNameKey` et l'éligibilité sont-ils décidables et testables tels qu'écrits, et les cas nommés donnent-ils
  l'attendu écrit (fais le calcul à la main, étape par étape) ? La route `supplier-candidates` (rôle, bornage, contacts
  archivés, ordre) ; AC12-bis : chaque motif de refus a-t-il un test et un libellé ; l'« effet sur la création manuelle »
  est-il exact au regard d'`Iban::new` ? E2E : défauts des répertoires, `seedTestState`, copies uniques, bloc
  `describe.serial`, indépendance. Textes, i18n ×4, manuels, CHANGELOG, `api-external.md`. Écrans et rôles.
- **F — Faisabilité, sécurité, verrous et naissance de la vue (adversaire de la conception)** : le test déterministe
  d'AC19 (A sans valider sur I1, B sur I1 + 1 sous 5 s) prouve-t-il ce qu'il prétend, et M-c11 le fait-elle rougir
  (verrous posés par un `SELECT … FOR UPDATE` sur une clé absente d'un index unique, puis par l'insertion) ? Le test
  d'AC24 « rattachement pendant un détachement non validé » est-il montable avec `attendre_une_requete_en_cours`, et
  quelle issue prévoir (attente puis succès, ou interblocage) ? `find_by_supplier_invoice_id_scoped` en exécuteur
  générique dans l'instantané de `get_settlement_view` ; la lecture de l'écriture dans la transaction de
  `get_journal_entry`. AC12-bis : contournement par normalisation (espaces insécables, minuscules, référence QRR à
  espaces) ; la référence « forme QRR » refusée avec un IBAN ordinaire peut-elle refuser une référence libre légitime
  à 27 chiffres ? La route `supplier-candidates` : IBAN jamais dans l'URL ni dans les journaux, `REPLACE(UPPER(…))`
  sans index, coût. CSP finale et bornes d'AC6-bis. Contrôle pur avant l'archivage : l'ordre de `process_one_file`
  (doublon, décodage, contrôle, archivage, insertion) reste-t-il juste, et la réactivation n'est-elle pas affectée ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. Dis pour chaque finding s'il vient de la **remédiation P2**, de la remédiation P1 ou de la conception d'origine. ⛔ **La
liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
