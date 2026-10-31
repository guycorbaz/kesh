# Prompt — validation P3 de la sous-fiche v016-2b (l'écran, #816), deux lentilles, Sonnet

Tu valides la **sous-fiche** `v016-2b-releve-manuel-ecran.md` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). Elle est
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

1. La sous-fiche `_bmad-output/implementation-artifacts/v016-2b-releve-manuel-ecran.md`, **en entier**.
2. La fiche **parente** `_bmad-output/implementation-artifacts/v016-2-releve-manuel.md`, en entier : contexte, décisions,
   contrat de l'API (corps, réponse de l'aperçu, codes d'erreur, ensembles fermés, ordre des refus), § *Découpage* et sa
   table de correspondance, limites, liste de la release, questions Q1 à Q8, Change Log (entrées « Validation P1 » et
   « Validation P2 et découpage »).
3. L'autre sous-fiche (`v016-2a-releve-manuel-backend.md`), pour
   la frontière entre les deux.
4. Le registre commun `_bmad-output/implementation-artifacts/v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-18).
5. Les rapports `/home/gcorbaz/devel/kesh-gate-logs/v0162-validate-p2-{R,F}.md`, et le diff de la remédiation :
   `git log --oneline -8`, puis `git show <commit « remédiation de la validation P2 »> -- _bmad-output/`.
6. L'issue, **en lecture seule** : `gh issue view 816 --json title,body,comments` ; le mandat des données à garder,
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
7. Le code : `frontend/src/lib/features/bank-import/` (dont `BankImportList.svelte`, `DeleteBankImportAction.svelte`,
   `csv-line-error-label.ts` et son test, `bank-import.api.ts`, `bank-import.types.ts`),
   `frontend/src/lib/features/journal-entries/balance.ts` et ses usages, `frontend/src/routes/(app)/bank-import/`,
   `frontend/src/routes/(app)/supplier-invoices/import/+page.ts`, `frontend/src/routes/(app)/audit-log/`,
   `frontend/src/lib/shared/i18n-*.test.ts` (dont `i18n-keys.test.ts`), `frontend/scripts/lint-i18n-ownership.js`,
   `frontend/tests/e2e/helpers/api-fixtures.ts`, `reconciliation-amount-due.spec.ts` ; `crates/kesh-i18n/locales/` ;
   `crates/kesh-core/src/bank_imports.rs` et `crates/kesh-import/src/camt053/mod.rs` (pour la conduite « la banque passe
   au CAMT ») ; `docs/manual/fr/user-manual.tex` et son PDF, aplati **vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ; `admin-manual.tex:82` ; `CHANGELOG.md` ;
   `README.md:41`.
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
- **L'écran contre le contrat de la 2a** : chaque règle « *Enregistrer* inactif » est-elle décidable par un test et
  compatible avec le serveur (ensembles fermés, asymétrie client / serveur écrite) ? La correspondance des rangs
  (lignes vides retirées) tient-elle avec les rangs de `BANK_STATEMENT_INVALID` et de `BANK_STATEMENT_DUPLICATE_LINES` ?
  La généralisation d'`isValidAmount` laisse-t-elle intacts ses usages du journal ? `toFixed(2)`, `[...s].length` après
  `trim`, aperçu « réussi », cases envoyées dans le corps ?
- **i18n** : la table des `reason` et sa famille (`MOTIFS_DYNAMIQUES`, `CARDINALITES`), la propriété des clés, les gardes
  `i18n-*.test.ts` nommées ; frontière avec les clés serveur posées par la 2a.
- **Le parcours Playwright (AC-26)** : faisable avec les aides existantes (et le paramètre `isPrimary`) ? dates dans la
  fenêtre de proposition et l'exercice ouvert ? compte propre à l'exécution, IBAN valide ? journal d'audit ouvert au
  Comptable ?
- **Les mutations** (M13 à M15, M19) : chacune rougit-elle le test annoncé ? Une régression probable de l'écran reste-t-elle
  sans mutation ?

### Lentille F — les textes, l'écran et ce qu'ils affirment

- **Le manuel utilisateur (AC-27)** — axe obligatoire : la sous-section prescrite promet-elle quoi que ce soit que le code
  ne tiendra pas ? La conduite « la banque passe au CAMT » suit-elle la clé réelle (premier identifiant non vide) ?
  L'inventaire des phrases à récrire est-il complet, au `.tex` **et** au PDF aplati (`CAMT`, `CSV`, `import`, `fichier`,
  `solde de clôture`) ? La FAQ ? `admin-manual.tex:82` ? Le PDF régénéré au commit du `.tex`, macros inchangées ?
- **Les libellés (AC-23, AC-24)** : l'inventaire fermé de la liste, du détail et de la boîte de suppression est-il
  complet ? Le message « fichier » du refus « relevé identique » est-il bien écarté de l'écran ?
- **La recette visuelle (AC-25)** : chaque état touché est-il listé ?
- **Le CHANGELOG *Ajouté* (AC-29-bis) et le README (AC-30)** : cohérents avec ce que la 2a pose (en-tête récrit, ligne
  de feuille de route) ?
- **Les décomptes et numéros de ligne** : recompte-les depuis la source.
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; aucune donnée réelle
  (exemples, IBAN de l'E2E synthétiques) ; périmètre minimal.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p3-F.md` (lentille F). Il contient :

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
