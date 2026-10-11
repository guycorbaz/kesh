# Prompt — validation P4 **ciblée** de la spec, Story 26-1 (import du CSV UBS tel quel)

*Versionné le 2026-10-11. **Une seule lentille, Haiku**, contexte frais, lecture seule. Passe ciblée de fin de boucle
(CLAUDE.md § « La passe ciblée » ; décision D6 de la rétro 25 : Haiku réservé à la passe ciblée). Trend : P1 1 HIGH /
7 MEDIUM → P2 5 MEDIUM → P3 2 MEDIUM, tous nés de la remédiation précédente.*

Dépôt `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1`. **Ton seul périmètre est le commit `d73d07d8`** (remédiation
de la validation P3). Lis-le **aplati** : `git show d73d07d8 -- _bmad-output/implementation-artifacts/` (un seul commit,
pas une série — pas de confusion de numéros de ligne entre commits). Fichiers touchés : la fiche
`_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md`, le registre `epic-26-choix-autonomes.md`
(C-26-1-25 à 27 ajoutés, corps de C-26-1-5, 14, 15 réécrits), le prompt P3 (note de correction), `sprint-status.yaml`.
Rapports de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p3-{R,F}.md` — à lire **après** le diff.

## La lentille — « chasseur de régression de la remédiation P3 »

Question unique : **la remédiation P3 a-t-elle introduit un défaut, ou laissé une contradiction, dans ce qu'elle a
écrit ?** Pas de revue de la story entière. Axes, dans cet ordre :

1. **`whole_debit_column` (AC3, AC14-7-ter, C-26-1-25)** : la définition « parmi les lignes dont le montant Débit a été
   lu, `d` = débits non nuls, `e` = en défaut ; vrai ⇔ `d ≥ 2` et `e = d` » est-elle cohérente avec **chaque** cas limite
   écrit en AC14-7-ter (aucun débit, débits tous nuls, un seul débit, deux en défaut, profil `Negative`, date illisible,
   débit illisible) ? Un cas donne-t-il un attendu contraire à la définition ? `RowParse` / `DebitObservation` : la
   correspondance avec les quatre issues de `parse_row` (colonne unique, ligne de crédit, échec avant le montant, débit
   illisible → `NotRead`) est-elle complète ? Le test existant `parses_corrupt_profile_with_both_amount_and_split_prefers_split`
   (`crates/kesh-import/src/csv/parser.rs` l. ≈ 1036-1068) est-il bien nommé comme appelant à adapter ?
2. **Manuel (AC18)** : la fiche ne fait **plus** réécrire « Import avec un profil existant » (à la 26-3), mais fait
   réécrire « Détection des doublons ». Reste-t-il dans la fiche (AC, tâches, mutations, Dev Notes, conflits) une phrase
   qui fait encore réécrire la première, ou qui renvoie à une procédure que la 26-1 n'écrit plus ? La réécriture
   prescrite de « Détection des doublons » est-elle vraie du code — `dedup_key_scalar`
   (`crates/kesh-core/src/bank_imports.rs` l. ≈ 337-356, et `normalize_reference_fallback`), les `data-testid` cités de
   `frontend/src/lib/features/bank-import/BankImportUpload.svelte` ? Les motifs `grep` du PDF ont-ils un `.` pour chaque
   apostrophe ?
3. **`i18n-keys.test.ts` (AC3-bis)** : les noms et lignes cités (`MOTIFS_DYNAMIQUES`, `CARDINALITES`,
   `SITES_GABARIT_ATTENDUS`, `ATTENDU.sitesGabarit: 10`, les titres « 8 préfixes » / « 10 sites ») existent-ils tels
   quels dans `frontend/src/lib/shared/i18n-keys.test.ts` ? `grep -nF` chacun.
4. **AC9 / `#[allow(dead_code)] mod common;`** : l'appel `common::migrations_before(20261011000001,
   "bank_profiles_debit_sign")` correspond-il à la signature réelle (`crates/kesh-db/tests/common/mod.rs` l. ≈ 75) ?
   L'argument rendu est-il bien le **nombre** attendu par la fonction **locale** `apply_migrations_up_to` de
   `migrations_upgrade_path.rs` ?
5. **Propagation** : chaque symptôme corrigé en P3 a-t-il encore une copie ? Greper la **valeur** sur la fiche, le
   registre et les prompts : `index_of_migration`, `importez à nouveau`, `relais dans`, `M1-M16` (hors note `[sic]`),
   `d ≥ 1`, `Les { $count }` (ancien message), `lignes vides non comptées` hors *Précisé* du CHANGELOG. Les décomptes
   écrits (21 AC, 28 mutations M1-M28, bilan P3 de 20 lignes, C-26-1-1 à 27) se **recomptent** à la source.
6. **Données personnelles** : aucun chiffre du relevé réel (somme, soldes, nombres de mouvements, de sorties,
   d'entrées, de lignes) ne doit apparaître dans le diff ni dans ton rapport. Le vrai fichier est
   `/home/gcorbaz/kesh-test/transactions.csv` : si tu le lis, c'est pour une comparaison **qui n'affiche rien** (sortie
   limitée à `fichier:ligne: <catégorie>`).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. ⛔ **Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée** (CLAUDE.md,
§ Haiku-specific guardrails) : un finding « X n'a pas été écrit » sans `grep -nF` sur le **fichier final** ne compte pas.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p4-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et `scripts/regen-test-schema.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un
script du dépôt, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
