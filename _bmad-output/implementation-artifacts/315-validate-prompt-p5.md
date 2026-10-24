# Prompt — validation P5, Story 31-5 (quatre défauts courts) — passe ciblée

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, *regression
hunter* sur la spécification, **Sonnet**, contexte frais, braquée sur **le seul diff de la remédiation P3**. Lecture seule
stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-5`, branche `story/31-5`. Fiche :
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-22). Règles utiles :
`CLAUDE.md` § *Review Iteration Rule*, *La passe ciblée* (une boucle ne se clôt que sur une passe sans finding au-dessus
de LOW), *Sonnet-specific guardrails*, *Propagation post-patch*, *Recompter ses propres comptes rendus*, *Priorités des
défauts*.

## Le périmètre : le diff aplati de la remédiation P3, et rien d'autre

```sh
git diff 3feaefff HEAD -- _bmad-output/implementation-artifacts/31-5-defauts-courts.md \
  _bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md
```

C'est **un seul commit** (`eec02e1f`), lu comme **un seul diff** `ee140ef5 → eec02e1f` : ne lis pas de séquence de
commits, ne raisonne pas sur des numéros de ligne de hunks intermédiaires. Pour situer une ligne, lis la fiche telle
qu'elle est (`sed -n`, `grep -nF`). Les rapports P3 qui ont motivé la remédiation sont
`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p3-R.md` et `…-p3-F.md` ; l'entrée « Validation P3 » du Change Log de
la fiche dit le sort de chaque finding. Ne les prends pas pour argent comptant.

**Ce que la P3 a changé** :

1. **F1 = R4** — la constante `VAT_SETTING_ACCOUNT_TYPES` de `kesh-db` est désormais **employée par la route**
   `crates/kesh-api/src/routes/company_invoice_settings.rs` (trois arguments de `validate_account`, l. 337-364) ;
   précédent `designated_account_id` cité ; fichier ajouté aux *Fichiers touchés* et aux *Risques* ; C-31-5-19 annoté.
2. **F2** — les libellés d'écriture en français sont tracés par **#712** (*Hors périmètre*, *Origine*, C-31-5-17).
3. **Budget** — 143 ouvertes, 138 au tag, trois retraits manquent (*Origine*, C-31-5-16).
4. **LOW** — test `kesh-db` dans `crates/kesh-db/tests/company_invoice_settings_repository.rs` ; prémisse 18-1a/18-1b
   recitée ; définitions de `VatRoleAccounts`, `VatSettingAccountTypes`, `VAT_SETTING_ACCOUNT_TYPES` (C-31-5-22) ;
   `INSERT IGNORE` l. 1123 ; ordre des trois lectures **permuté** (`VatRecoverable`, `VatPayable`, `VatSettlement`,
   C-31-5-21) ; nuance de `reference_score` ; replis `&format!(…)` ; ordre de T4 ; assertion « trois réglages `NULL` »
   de l'AC2.

## La lentille — *regression hunter* sur la spec

Motif mesuré : la sévérité se déplace vers ce qu'on vient d'écrire. Tu cherches ce que **cette** remédiation a
introduit ou laissé derrière elle :

1. **Chaque affirmation NOUVELLE se vérifie au code**, sortie copiée. Au minimum :
   - la route importe `AccountType` (l. 25-26) et le module `company_invoice_settings` de `kesh-db` (l. 28) :
     `sed -n 20,30p crates/kesh-api/src/routes/company_invoice_settings.rs` ;
   - `validate_account` prend `expected: AccountType` par valeur (l. 152-158) et ses trois appels de TVA sont aux
     l. 337-364 avec `Liability`, `Asset`, `Liability` ;
   - `AccountType` dérive `Copy` et implémente `Type`/`Decode` pour MySQL (`crates/kesh-db/src/entities/account.rs:11-66`) ;
   - `designated_account_id` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:495-520`) dit bien « Mêmes
     exigences que sa validation » ;
   - `INSERT IGNORE INTO company_invoice_settings` de `insert_with_defaults_in_tx` est l. 1123 ;
   - `lock_designated_accounts_in_tx` (l. 865-916) verrouille par identifiant croissant ;
   - `18-1a-comptes-tva.md:109-110` et `18-1b-comptabilisation-ventes.md:122-125` disent ce que la fiche leur attribue ;
   - `matching.rs:242` (contenance) et `:247-254` (préfixe commun, 0.5) ;
   - `errors.rs:52-59` (`t_args` rend `default.to_string()` sans bundle) et `errors.rs:3565-3567` (repli `format!`) ;
     `textes_coherents.rs:774-805` (G9 compare un littéral) ;
   - `onboarding_path_b_e2e.rs:306-312` (`settings_rows == 1`, seule assertion actuelle sur les réglages) ; les noms de
     colonnes `default_vat_payable_account_id`, `default_vat_recoverable_account_id`, `default_vat_decompte_account_id`
     (`crates/kesh-db/src/entities/company_invoice_settings.rs`) ;
   - `company_invoice_settings_postable_e2e.rs:150-170` (table des champs de TVA et de leurs types) ;
   - `crates/kesh-db/tests/company_invoice_settings_repository.rs:34-35` (montage du plan PME) ;
   - le recompte : `gh issue list --state open --limit 500 --json number --jq length` et `gh issue view 712`.
2. **Cohérence de la permutation de l'ordre** : chaque site de la fiche et du registre qui énonce l'ordre des trois
   lectures ou des verrous dit-il désormais `VatRecoverable`, `VatPayable`, `VatSettlement` (1171, 2200, 2206), hors
   entrées **datées** du Change Log et annotations qui citent l'ancien ordre comme tel ? Le raisonnement des Dev Notes
   (ordre opposé retiré au plan standard, possible hors plan standard) tient-il ?
3. **Cohérence de la règle de type unique** : reste-t-il un site qui parle d'une « jumelle », d'une route « non
   touchée », d'une règle « qui n'existe qu'à la route » comme d'un état prescrit ? La mutation M7a et le cas (ii) du
   test `kesh-db` restent-ils justes maintenant que la constante est partagée ? Le passage de « cinq modules » au
   *Découpage préventif* tient-il avec un fichier de route de plus ?
4. **Budget** : la chaîne 143 → 142 → 138 et « trois retraits » se recoupent-elles partout (fiche, registre, sprint-status
   `_bmad-output/implementation-artifacts/sprint-status.yaml`, clé `31-5-defauts-courts` et ligne 1) ? Reste-t-il un
   conditionnel (« si elle s'ouvre », « non ouverte ») ailleurs que dans une entrée datée ?
5. **Propagation** : grep des symptômes corrigés sur la fiche et le registre — `137`, `141`, `142`, `non ouverte`,
   `soumise au`, `jumelle`, `mod tests`, `2200, 1171`, `1126`, `voulu`, `deux fichiers de route` — et tri de chaque
   occurrence (légitime et datée, ou résidu).
6. **Manuel** : la remédiation a-t-elle changé une prescription de manuel (`docs/manual/fr/user-manual.tex`, AC3, AC7,
   AC10) ? Sinon, le dire.

## Ce que tu rends

Findings `P4-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche ou du registre, site du dépôt, preuve
(commande **et** sortie copiée), correction proposée. Pour tout finding qui affirme l'**absence** d'un texte ou la
**présence** d'un résidu, la preuve est un `grep -nF` sur le fichier tel qu'il est (règle *Sonnet-specific
guardrails*), pas une lecture du diff. Un bilan d'en-tête **cohérent avec le corps**, recompté sur la liste. ⛔ **La liste
des axes exercés ET non exercés** (1 à 6) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au
développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet** dans
`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p5.md` (seul fichier que tu écris, par ton outil d'écriture de fichier,
jamais par une redirection du shell). Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, **aucune redirection vers un fichier**
(`>`, `>>`, `tee`, `-o`). Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`,
`head`, `awk`, `wc` vers la sortie standard.
