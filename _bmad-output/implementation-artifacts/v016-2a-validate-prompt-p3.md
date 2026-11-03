# Prompt — validation P3 de la sous-fiche v016-2a (le backend, #816), deux lentilles, Sonnet

Tu valides la **sous-fiche** `v016-2a-releve-manuel-backend.md` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). Elle est
issue du **découpage**, à la validation P2, de la Story v016-2 — la saisie manuelle d'un relevé bancaire, pour un compte
dont la banque ne fournit ni CAMT.053 ni CSV — en `v016-2a` (backend) et `v016-2b` (écran). Un relevé saisi est un
import de `source_format = 'MANUAL'` créé par la même fonction de dépôt que l'import ; une migration relève
`kesh_version_min_required` à `0.17.0` (release **0.17.0**, première candidate `v0.17.0-rc1`) ; les données de
l'installation `v0.16.1-rc1` du NAS se gardent.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel`, partie de `c5101b07` (le code et les
manuels sont ceux de ce commit ; la branche ne touche que `_bmad-output/`). Langue du rapport : **français**. Modèle :
**Sonnet**. Passe **complète** — le découpage a réécrit la fiche. **Contexte frais** : ne t'appuie sur aucune passe
antérieure, ni sur ce que la fiche dit d'elle-même ; les rapports P1 et P2 se lisent pour savoir ce qui a été remédié,
non comme preuve que le remède est juste.

## À lire

1. La sous-fiche `_bmad-output/implementation-artifacts/v016-2a-releve-manuel-backend.md`, **en entier**.
2. La fiche **parente** `_bmad-output/implementation-artifacts/v016-2-releve-manuel.md`, en entier : contexte, décisions,
   contrat de l'API (corps, réponse de l'aperçu, codes d'erreur, ensembles fermés, ordre des refus), § *Découpage* et sa
   table de correspondance, limites, liste de la release, questions Q1 à Q8, Change Log (entrées « Validation P1 » et
   « Validation P2 et découpage »).
3. L'autre sous-fiche (`v016-2b-releve-manuel-ecran.md`), pour
   la frontière entre les deux.
4. Le registre commun `_bmad-output/implementation-artifacts/v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-18).
5. Les rapports `/home/gcorbaz/devel/kesh-gate-logs/v0162-validate-p2-{R,F}.md`, et le diff de la remédiation :
   `git log --oneline -8`, puis `git show <commit « remédiation de la validation P2 »> -- _bmad-output/`.
6. L'issue, **en lecture seule** : `gh issue view 816 --json title,body,comments` ; le mandat des données à garder,
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
7. Le code : `crates/kesh-db/migrations/20260504000001_bank_imports.sql` et suivantes `bank_*`,
   `crates/kesh-db/migrations.sha384`, `crates/kesh-db/src/entities/bank_import.rs`, `repositories/bank_imports.rs`,
   `bank_transactions.rs`, `bank_accounts.rs` (`BALANCES_SQL`), `closing_checks.rs`, `post_restore.rs`, `backup.rs` ;
   `crates/kesh-db/tests/migrations_upgrade_path.rs`, `migrations_fresh_install.rs`, `test_schema_guard.rs`,
   `bank_accounts_repository.rs` ; `crates/kesh-core/src/bank_imports.rs` ; `crates/kesh-import/src/camt053/mod.rs` ;
   `crates/kesh-api/src/routes/bank_imports.rs`, `bank_accounts.rs` (`BodyExtractor`), `admin.rs`, `lib.rs`, `errors.rs`,
   `audit_labels.rs`, `admin_backup/manifest.rs` ; les tests-gardes `bank_account_closed_inventory.rs`,
   `audit_route_registry.rs`, `audit_label_registry.rs`, `onboarding_audit_e2e.rs` (`FAIL_TRIGGER`),
   `admin_full_import_e2e.rs`, `admin_full_export_e2e.rs` ; `docs/api-external.md` ;
   `docs/manual/fr/admin-manual.tex` (§ *Sauvegarde et restauration*, § *Passer à la 0.16.1*, l. 2966, l. 3200-3360) et
   son PDF, aplati **vers la sortie standard seulement** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` ;
   `README.md` (l. 41, 49, 235) ; `kesh-style.sty` ; `docs/ci.md`.
8. Pour les règles : `CLAUDE.md` du worktree (*Migration breaking policy* P1 à P8, *Un appariement automatique propose,
   il ne crée jamais*, *Règle de splitting préventif*, *Recette visuelle*, *Issue Tracking Rule*, liste de contrôle
   pré-release), `docs/agents-orchestration.md`, `CHANGELOG.md` § `[0.16.1]`.

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Axes obligatoires des deux lentilles

1. **La conservation des AC à travers le découpage.** Chaque AC de la fiche d'origine (état remédié de la P2, **juste avant** le
   découpage, conservé hors dépôt : `/home/gcorbaz/devel/kesh-gate-logs/v0162/v016-2-avant-decoupage.md`, en lecture ;
   il a été obtenu en appliquant à la fiche de `7bf2a0b1` le script de remédiation
   `/home/gcorbaz/devel/kesh-gate-logs/v0162/remed-p2-step1.py`, puis le découpage par `remed-p2-step2.py` — lis-les
   s'il le faut, ne les exécute pas)
   est-il dans **une seule** sous-fiche, **sans perte ni altération** de son texte (hors l'AC-29, dont le point 2 devient
   l'AC-29-bis de la 2b) ? La table de correspondance de la parente le dit-elle exactement ? Un AC qui dépend de l'autre
   sous-fiche (clés i18n serveur et d'écran, codes et `reason` consommés par l'écran, PDF touchés par les deux, CHANGELOG
   partagé) est-il cohérent des deux côtés, sans trou ni double propriétaire ? Les tâches, les mutations, les gates, les
   fichiers touchés et la liste « À recontrôler » de la sous-fiche couvrent-ils tous ses AC, et seulement eux ? La
   sous-fiche est-elle **livrable et testable seule** (2a) ou après la seule 2a (2b) ?
2. **La remédiation P2.** Chaque passage ajouté ou récrit par la remédiation P2 et qui relève de cette sous-fiche (ou de
   la parente qu'elle invoque) : chaque `fichier:ligne` nouveau dit-il ce qu'on lui fait dire ? Chaque remède tient-il au
   code, sans contradiction avec un autre critère ? Un finding P2 déclaré « appliqué » l'est-il entièrement (copies du
   symptôme ailleurs) ? Les annotations du registre (C-v016-2-6, C-v016-2-13, C-v016-2-17, C-v016-2-18) sont-elles
   justes ?

### Lentille R — la règle et sa preuve

- **Les prémisses** : chaque `fichier:ligne` cité par la sous-fiche, la parente et le registre dit-il ce qu'on lui fait
  dire ?
- **Le modèle et l'API** : un relevé `MANUAL` est-il indiscernable d'un import pour le rapprochement, la suppression, la
  tuile, K3, l'export et la sauvegarde ? Le type d'entrée en `Option<String>` et l'extracteur rendent-ils **toute** faute
  de forme en `BANK_STATEMENT_INVALID` avec son `field` et son `reason`, et les ensembles fermés sont-ils complets et
  atteignables ? L'ordre des refus tient-il avec la transaction et la garde du compte clos ? La limite de corps est-elle
  juste ?
- **La migration et la version** : P1, P2, P2-bis (dix crates en `0.17.0` au même commit), P5, P6, P7 (compteur 21 → 22),
  P8, `migrations.sha384`, squash ; tests qui figent `0.14.0` — sémantique, non seule valeur ; tableau des propriétaires
  des `0.16.1` complet (`grep -rn "0\.16\.1"` sur tout le dépôt) ?
- **Les données à garder** : AC-2-bis et AC-3-bis prouvent-ils la montée sans perte et l'import d'une sauvegarde 0.16.x,
  par les moyens prescrits (patrons cités) ? Aucune procédure n'écrit-elle sur la base du NAS ?
- **Les gardes** : `audit_label_registry.rs` (les deux sens, `PAS_DES_CODES`, « neuf »), `audit_route_registry.rs`
  (tous les compteurs figés, dont `sans_ecriture`, et la phrase du manuel), `bank_account_closed_inventory.rs` (dont le
  test étendu) : la sous-fiche prescrit-elle tout ce qui rougira ?
- **Les mutations** (M1 à M12, M16 à M18, M20 à M22) : chacune rougit-elle le test annoncé, pour la raison annoncée ?
  M22 et son déclencheur ; M16 et M17 et la règle P8.

### Lentille F — les textes et ce qu'ils affirment

- **Le manuel d'administration** — axe obligatoire : § *Passer à la 0.17.0* (la migration, la sauvegarde complète
  d'avant la mise à jour, le retour par elle seule, la sauvegarde 0.16.x importable), `:2270`, `:2966`, `:3220`, `:3238`,
  `:3349`, `:3351` ; le PDF régénéré au commit de son `.tex` sans toucher aux macros : le prescrit est-il complet et juste,
  au `.tex` **et** au PDF aplati ?
- **Le CHANGELOG `[0.17.0]`** (AC-29) : renommage, en-tête récrit, phrase « aucune migration » supprimée, note sur la
  candidate `0.16.1-rc1`, `0.16.1` des lignes 11 à 19 ; frontière avec l'entrée *Ajouté* de la 2b (AC-29-bis).
- **`docs/api-external.md` (AC-28)** : routes, corps, codes, ensembles fermés, `sourceFormat`, compte clos, rejeu.
- **`README.md:49` et `:235`**, `user-manual.tex:3265` : prescrits juste ?
- **La liste de la release** de la parente, au regard de ce que la 2a livre (blocs A et B, PDF, macros, URL G19).
- **Les décomptes** (87 migrations, 46 sites P6, compteurs du registre des routes, 22 exemptions, numéros de ligne) :
  recompte-les depuis la source.
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; aucune donnée réelle ;
  périmètre minimal.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0162a-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0162a-validate-p3-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `mariadb`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
