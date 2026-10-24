# Prompt — revue de code P1 de la Story 32-1a (tests et commentaires), trois lentilles, Sonnet

Tu relis le développement de la clé **32-1a** de la Story 32-1 de Kesh (comptabilité suisse) : *tests et
commentaires* — **#716** entier (huit constats LOW de la revue de la 31-5 : B1, B2, B3 = E3, E1, E2, E5, E6, E7) et
**E-9 (1)** et **E-9 (2)** de **#711** ; l'angle mort de **E-9 (3)** est seulement **écrit** (l'AC-a11 est retiré,
renvoyé à #727). Worktree `/home/gcorbaz/devel/kesh-32-1a`, branche `story/32-1a`, partie de la fiche validée
`01b9a58b` (= `story/32-1`). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-1a diff story/32-1...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` — **partie A** (AC-a1 à AC-a13, tâches T-a0 à T-a8),
§ *Dev Notes* (*32-1a — remarques de conception*, *Inventaire des sites non résolus*), et la sous-section **32-1a** du
Dev Agent Record — et le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à
C-32-1-38, plus les entrées éventuelles du développement). Le Dev Agent Record déclare des gates, des rouges et des
mutations : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/321a-*.log` et `/home/gcorbaz/devel/kesh-gate-logs/321-mut-*.log` (lecture
seule). La 32-1a n'ajoute ni ne modifie aucun contrôle visible : pas de capture d'écran attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
  Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris),
  ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
  Aucune commande `gh` qui écrit ; `gh issue view 716|711|727|725` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence** (« le site X n'est pas corrigé », « le test Y manque », « le commentaire ne dit
  pas Z ») s'appuie sur une sortie **`grep -nF`** copiée ; toute affirmation de présence, sur une sortie `grep -nF` ou
  `sed -n` copiée.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/tests/textes_coherents.rs` : la regex `DATE_ISO` de `nombres_isoles` et son test neuf. La regex
   bornée ouvre-t-elle un faux rouge sur un manuel ou un catalogue (une date réelle désormais lue comme compte) ou un
   vert muet ? Le test prouve-t-il ce que dit son nom (positifs **et** négatifs) ?
2. `crates/kesh-api/src/routes/invoice_pdf_service.rs` : le commentaire du préfixe de la QR (cas d'un PDF **figé**) —
   dit-il vrai sur le code (où le PDF se fige, où le rappel se régénère, ce que lit le rapprochement) ? Le test
   `qr_message_prefix_is_translated_in_all_four_locales` : passerait-il à vide (clé brute, repli français) ?
3. `crates/kesh-db/src/repositories/company_invoice_settings.rs` : le doc-comment de `lock_vat_role_accounts_in_tx`
   (séquence complète des verrous) — vrai contre `insert_with_defaults_in_tx`, `chart_designated_accounts`,
   `designated_account_id` et `crates/kesh-core/assets/charts/pme.json` ? N'énumère-t-il pas comme exhaustive une
   liste qu'il ne tient pas ?
4. `crates/kesh-db/tests/company_invoice_settings_repository.rs` : les deux tests `designate_vat_*` — montage en
   transactions distinctes (un verrou peut-il s'attendre lui-même ?), assertions de montage (valeurs légitimes,
   distinctes des rôles, colonnes `NULL` après création), décodage des colonnes (`account_type` en `String`, `version`
   en `i32`). DRY avec les helpers existants du fichier (`pme_company_with_chart`, `read_vat_role_accounts`) ?
5. `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts` : le cas « un clos suivi d'un ouvert ».
6. `crates/kesh-api/tests/audit_log_e2e.rs` : `catalogues_charges()` employée par `spawn_app_de` (un seul
   `I18nBundle::load`) ; les deux cas neufs du tableau (`[Cas; 12]`).
7. `crates/kesh-api/tests/admin_auto_backup_e2e.rs` : la sonde de `TestEnv` (`mesures`, `sonde()`), `large()` et
   `avec()` inchangés dans leur effet ; l'assertion neuve de `dossier_auto_inutilisable` et son doc-comment.
8. `crates/kesh-api/src/auto_backup.rs` : doc-comments seuls dans le `mod tests` (`code()`, `argument_de`, `corps()`,
   `gardes_possedes`). **Aucune ligne de code** ne doit avoir changé : vérifie-le au diff. Les doc-comments ajoutés
   introduisent-ils, dans le texte que relisent ces gardes, un jeton exigé ou interdit (`try_lock_owned()`,
   `backup_lock.lock(`, `backup_lock.try_lock(`, `auto_backup::spawn(`, une ligne `#[cfg(test)]` seule) ?

## Lentille E — Edge cases

1. `DATE_ISO` : `NNNN-MM-JJ` suivi d'un chiffre, d'une lettre, d'un `T` (horodatage), en fin de ligne ; mois `00`,
   `13`, jour `00`, `32` ; `2026-02-30` (limite écrite) ; une date dans un manuel ou un catalogue réel qui aurait
   cessé d'être reconnue (cherche-les : `grep -rnE '[1-9][0-9]{3}-[0-9]{2}-[0-9]{2}' docs/manual/fr/*.tex
   crates/kesh-i18n/locales` et raisonne sur celles dont le mois ou le jour sort des bornes).
2. `designate_vat_*` : que prouvent-ils si `insert_with_defaults_in_tx` venait à poser une colonne de TVA ? si un rôle
   de TVA manquait au plan ? si `designate_vat_accounts_in_tx` incrémentait `version` ? si l'`UPDATE` ne visait aucune
   ligne (`WHERE company_id`) ? Une mutation plausible non jouée survivrait-elle ?
3. `latestLaterClosed` : le cas neuf tient-il l'exercice **ouvert intercalé** (A clos, B ouvert, C clos) ? Sinon, est-ce
   un trou réel ou hors de l'AC-a5 ?
4. `audit_log` : l'ordre des refus au-delà des deux dates (une date invalide **et** un code trop long ; `entityType`
   trop long **et** `action` trop long) — la fiche fige-t-elle plus que ce que le test tient ?
5. `dossier_auto_inutilisable` : le compteur est-il partagé entre deux tours, deux tests (parallélisme de nextest) ?
   L'assertion à zéro pourrait-elle passer pour une autre raison que la garde (tour refusé plus tôt, verrou pris) ?
6. **Mutations** M-a1a, M-a1b, M-a1c, M-a2, M-a3, M-a4 (et sa variante `de-CH` seul), M-a5, M-a6 : chaque mutation
   déclarée tuée l'est-elle par le test que la fiche nomme (journaux `321-mut-*.log`, `321a-mutations.log`) ? Le diff
   de la mutation est-il celui que décrit la fiche ? Deux runs sont annotés invalides (`321-mut-M-a1b-temoin-sans-
   mutation.log`, `321-mut-M-a2-run-invalide.log`) : la mutation a-t-elle bien été rejouée ensuite ? Raisonne sur le
   code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-a1 à AC-a13, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC-a1, AC-a8, AC-a10** : commentaires seuls ; contenu exigé par la fiche présent (`grep -nF`) ; aucune ligne de
  code des helpers lexicaux ni de `designate_vat_accounts_in_tx` changée (`git diff story/32-1...HEAD --
  crates/kesh-api/src/auto_backup.rs crates/kesh-db/src/repositories/company_invoice_settings.rs`, ne lire que les
  lignes `+`/`-` hors `//`) ; renvoi à #727 écrit ; `crates/kesh-db/tests/lexique/mod.rs` non touché.
- **AC-a2** : `grep -c 'I18nBundle::load' crates/kesh-api/tests/audit_log_e2e.rs` doit rendre **1**.
- **AC-a3** : quatre locales, « ni vide ni égal à la clé », « diffère du fr-CH » ; rouges M-a4 (quatre catalogues) et
  variante `de-CH` seul (journaux `321-mut-M-a4.log`, `321-mut-M-a4-de.log`) ; aucun repli Rust ajouté.
- **AC-a4** : deux tests, montage (t1)/(u)/(t2)/(t3) tel que la fiche le dit, comptes 2000/1170/2201, assertions de
  montage, `version` relue ; M-a1a/b/c tuées par `designate_vat_never_overwrites_a_set_value`.
- **AC-a5, AC-a6, AC-a7, AC-a9** : cas, mutations ou rouge constaté (`321a-rouge-AC-a7.log`), limite écrite.
- **AC-a11** : retiré — vérifie qu'aucun code des helpers ne change et qu'aucun module `lexique` n'est déclaré dans
  `auto_backup.rs` (`grep -nF 'mod lexique' crates/kesh-api/src/auto_backup.rs`).
- **AC-a12** : les chiffres du Dev Agent Record (nextest, Vitest, `check`, `lint-i18n-ownership`, `build`, E2E et liste
  nominative des échecs) correspondent-ils **aux journaux** `321a-gate-complet.log`, `321a-frontend-*.log`,
  `321a-e2e.log` ? Les échecs E2E sont-ils jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* ?
  Le gate et l'E2E ont-ils tourné **au dernier commit de code** (compare la révision en tête des journaux au
  `git log --oneline story/32-1..HEAD` : les commits postérieurs ne touchent que de la documentation) ?
- **AC-a13** : huit mutations, contre `321a-mutations.log` ; diff résiduel 0.
- **Décomptes** : « tests ajoutés » du Dev Agent Record recomptés aux deux bornes (`git show story/32-1:<f> | grep -cE
  '#\[(sqlx::|tokio::)?test'` contre le fichier à `HEAD` ; pour le Vitest, `grep -cE '^\s*it\('`) ; périmètre déclaré.
- **Messages de commit** : `git log story/32-1..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
  #[0-9]'` doit ne rien rendre ; chaque commit porte `refs #716, refs #711`.
- **Registre et statut** : la clé `32-1a-defauts-tests-commentaires` de `sprint-status.yaml` est à `review` avec un
  commentaire daté ; le Dev Agent Record n'écrit que la sous-section 32-1a ; aucune entrée CHANGELOG (la 32-1a n'a pas
  d'effet visible).

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321a-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff` — toute affirmation de présence ou d'absence s'appuie sur une sortie), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
