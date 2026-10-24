# Prompt — revue de code P1 de la Story 31-5 (quatre défauts courts), trois lentilles, Sonnet

Tu relis le développement de la **Story 31-5** de Kesh (comptabilité suisse) : quatre défauts courts avant la v0.15.0 —
**#707** (la démonstration désigne les trois comptes de TVA du plan, `kesh-db` + `kesh-seed` + la route des réglages),
**#589** (l'infobulle « Réouvrir » nomme le plus récent exercice clos), **#504** (la communication de la QR-facture suit
la langue du document), **#469** (les six refus du journal d'audit traduits). Worktree `/home/gcorbaz/devel/kesh-31-5`,
branche `story/31-5`, base `origin/main` = `e71cd351`. Le diff à relire, **aplati** (un seul diff, pas la suite des
commits) : `git -C /home/gcorbaz/devel/kesh-31-5 diff origin/main...HEAD`. Les commits de la fiche (validation P1 à P5)
font partie de ce diff : le code commence au commit `05f8a944`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` (AC1 à AC16, Dev Agent Record) et le registre
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-25 ; 23 à 25 au
développement). Le Dev Agent Record déclare des gates, des rouges et des mutations : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/315-*.log` (lecture seule). Les captures
d'écran sont jointes dans `/home/gcorbaz/devel/kesh-gate-logs/315-captures/` (PNG, thème clair, base de démonstration
synthétique ; valeurs des infobulles relevées au DOM dans `315-captures-notes.txt`) : ouvre-les.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, aucune
  redirection vers un fichier (`>`, `>>`, `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la
  sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni les scripts `315-*.sh` de `kesh-gate-logs`, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande), ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
  Aucune commande `gh` qui écrit ; `gh issue view 707|589|504|469` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, la lecture des PNG.
- **Toute affirmation d'absence** (« le site X n'est pas corrigé », « le test Y manque ») s'appuie sur une sortie
  `grep -nF` copiée.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-db/src/repositories/company_invoice_settings.rs` : `VatRoleAccounts`, `VatSettingAccountTypes`,
   `VAT_SETTING_ACCOUNT_TYPES`, `lock_vat_role_accounts_in_tx`, `lock_usable_role_account`,
   `designate_vat_accounts_in_tx`. Requêtes (index lu, `FOR UPDATE`, décodage `(i64, AccountType, bool)`), filtre,
   `COALESCE`, doc-comments (disent-ils vrai sur l'ordre des verrous et sur ce qu'ils ne garantissent pas ?).
2. `crates/kesh-seed/src/lib.rs`, `final_body` : position des deux appels, condition `invoice_settings_created`, rejeu
   sur interblocage, doc-comment. `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (ligne `seed_demo`) dit-il le code ?
3. `crates/kesh-api/src/routes/company_invoice_settings.rs` : comportement de la route inchangé ?
4. `crates/kesh-api/src/routes/invoice_pdf_service.rs` : résolution du préfixe, paramètre de `build_qrbill_inputs`,
   troncature à 140 caractères, marques d'isolation Fluent, commentaire du rapprochement.
5. `crates/kesh-api/src/routes/audit_log.rs` et `errors.rs` (`t_args` `pub(crate)`) : six sites, replis, variables.
6. `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` : `latestLaterClosed`, doc-comment, appel.
7. Tests ajoutés ou modifiés (`onboarding_e2e.rs`, `onboarding_path_b_e2e.rs`, `company_invoice_settings_repository.rs`,
   `invoice_frozen_pdf_e2e.rs`, `audit_log_e2e.rs`, `textes_coherents.rs`, `fiscal-years-page.test.ts`, le test unitaire
   de `invoice_pdf_service.rs`) : prouvent-ils ce que dit leur nom, ou pourraient-ils passer à vide ? DRY avec des
   helpers existants ? La retouche de `nombres_isoles` (date ISO) ouvre-t-elle un vert muet ?
8. Catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` : sept clés, traductions justes,
   variables identiques dans les quatre.

## Lentille E — Edge cases

1. `lock_vat_role_accounts_in_tx` : rôle absent, compte archivé, non imputable, type du rôle admis mais pas du réglage,
   deux comptes portant le même rôle (possible ?), plan `Independant` et `Association` (`crates/kesh-core/assets/charts/`).
   Chaque branche : quel test l'exerce, ou aucun.
2. `final_body` : ligne de réglages préexistante (`invoice_settings_created` faux) — atteignable ? effet d'une
   réinitialisation de démonstration puis d'un nouveau `seed-demo` (`reset_demo`, tables vidées) ? Rejeu après
   interblocage : idempotence réelle ? La mutation M4 déclarée **équivalente** : la démonstration tient-elle ?
3. `latestLaterClosed` : exercices de même `startDate`, exercice ouvert intercalé (A clos, B ouvert, C clos), un seul
   exercice, aucun clos postérieur ; cohérence avec la garde serveur `reopen` (`crates/kesh-db/src/repositories/fiscal_years.rs`).
4. Préfixe QR : locale sans clé (repli de `bundle.format`), facture à langue **forcée** et son rappel, numéro absent,
   numéro long (140), caractères hors jeu SIX. PDF figés antérieurs (AC10) : rien ne les refige ?
5. Refus du journal d'audit : les dix cas du test ; `dateFrom` et `dateTo` invalides ensemble (lequel parle ?) ; un code
   `entityType` trop long ; rendu sans bundle (repli `format!`, accolades) ; marques d'isolation.
6. Mutations M1 à M8 (M7a, M7b) : chaque mutation déclarée tuée l'est-elle par le test que la fiche nomme (journaux
   `315-mut-*.log`, `315-mutations.log`) ? Une mutation plausible non jouée survivrait-elle ? Raisonne sur le code,
   n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC16, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC1, AC2** : définitions, ordre des lectures, filtre du **réglage** (non `accepts_account_type`), route employant
  la constante (écart déclaré C-31-5-25 : acceptable ?), quatre cas du test `kesh-db`, test HTTP démonstration et
  production, rouge constaté (`315-rouge-707.log`).
- **AC3, AC7, AC10 — manuel** : `docs/manual/fr/user-manual.tex` (l. 185, le paragraphe après l. 205, la FAQ « Comment
  changer la langue d'une facture ? ») **et le PDF** `docs/manual/fr/user-manual.pdf`, contrôlé aplati
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, attention à l'apostrophe typographique et aux
  ligatures) : les trois phrases neuves y sont-elles, et disent-elles vrai sur le code ? Le message cité est-il
  exactement `error-configuration-required` ? La phrase « indique lequel rouvrir d'abord » est-elle désormais vraie ?
  `admin-manual.pdf` et `marketing-brochure.pdf` non modifiés (`git diff --stat origin/main...HEAD -- docs/manual`) ?
  Captures : *Administration → Facturation* (trois comptes de TVA remplis) et *Exercices comptables* (trois clos).
- **AC4-AC6** : renommage (quatre sites, `grep -rn "nearestLaterClosed" frontend/src` → zéro), aucune clé i18n de
  l'infobulle changée, inventaire de l'AC6.
- **AC8, AC9** : clé sans variable, quatre locales, rouges (a) en deux temps et (b) constatés (`315-rouge-504-*.log`).
- **AC11, AC12** : six sites, replis égaux au catalogue (G9, entrée `audit_log.rs`), test `de-CH` égal/différent, rouge
  (`315-rouge-469.log`).
- **AC13** : parité des catalogues, G9, `lint-i18n-ownership`.
- **AC14** : les chiffres du Dev Agent Record (nextest, Vitest, `check`, `build`, E2E et liste nominative des échecs)
  correspondent-ils **aux journaux** `315-gate-final-*.log` et `315-e2e.log` ? Les échecs E2E sont-ils jugés fichier par
  fichier contre `docs/testing.md` § *Les échecs attendus* ? Le gate et l'E2E ont-ils tourné **au dernier commit de
  code** (compare au `git log --oneline origin/main..HEAD` : les commits postérieurs ne touchent que de la
  documentation) ? Le premier run E2E (montage incomplet, `315-e2e-run1-montage-incomplet.log`) est-il déclaré tel ?
- **AC15** : mutations, contre `315-mutations.log`.
- **AC16** : rejoue les trois contrôles de la fiche (lecture seule) et compare.
- **CHANGELOG** : section `### Corrigé` de `[0.15.0]`, quatre entrées justes, placée avant `### Interne…`.
- **Messages de commit** : `git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte ses `refs`.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/315-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff` — toute affirmation de présence ou d'absence s'appuie sur une sortie), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
