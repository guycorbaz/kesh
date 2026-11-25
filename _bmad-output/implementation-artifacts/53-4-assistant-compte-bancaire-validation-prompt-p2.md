# Prompt — validation P2 de la fiche 53-4 « L'assistant ne réécrit pas le compte bancaire » (#1004), passe complète

Tu valides la **fiche** de la story **53-4** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), Epic 53 « Écritures justes »,
release visée **v0.21.0** (vendredi 2026-12-04). **Kesh est en production** : le Project Lead tient ses livres réels. Deuxième passe :
**complète**, contexte frais, modèle **Opus**. Langue du rapport : français. Tu relis ; tu ne corriges pas la fiche, tu rends des findings
(CRITICAL, HIGH, MEDIUM, LOW), chacun avec `fichier:ligne` copié et l'extrait cité.

Worktree `/home/gcorbaz/devel/kesh-53-4`, branche `story/53-4-assistant-compte-bancaire`, base `origin/epic/53-ecritures-justes` à
`4d540a6ea`. Fiche : `_bmad-output/implementation-artifacts/53-4-assistant-compte-bancaire.md` (statut `drafted`). Epic :
`_bmad-output/planning-artifacts/epic-53-ecritures-justes.md`.

**Contexte** : la P1 (Opus, rapport `/home/gcorbaz/devel/kesh-gate-logs/e534/validation-p1.md`, à lire en entier) a rendu 0 CRITICAL, 0 HIGH, 5 MEDIUM, 12 LOW ; la fiche a été remédiée (Change Log « validation P1 remédiée ») et les décisions du Project Lead du 2026-11-25 y sont écrites comme tranchées (Q1 P1 maintenu ; Q2 serveur inchangé, QR-IBAN toujours renvoyé ; Q3 blocage avec message clair). Ne rouvre pas ces décisions ; vérifie qu'elles sont appliquées sans contradiction.

**Le point le plus important** : la fiche affirme que le serveur est **juste** (absent = inchangé, `null` = effacé) et que le défaut est
**à l'écran seulement**, qui envoie toujours des valeurs vides ou présélectionnées ; elle ajoute que le **QR-IBAN** est aussi effacé.
Vérifie-le toi-même au code avant tout le reste. Cherche aussi un **quatrième champ écrasé** que la fiche aurait omis, et un cas où
le correctif d'écran **créerait** un défaut (valeur obsolète renvoyée, lien conservé qui contourne un refus du serveur).

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Recette visuelle*, *E2E (Playwright)*, *Review Iteration Rule* (dont *Inventorier les
   sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*), *Priorités des défauts*,
   *Issue Tracking Rule* (mots-clés de fermeture), *Migration breaking policy* (la story n'en porte aucune : vérifie).
2. `docs/agents-orchestration.md` §§ 2, 2.1, 3.
3. La fiche en entier, puis `_bmad-output/implementation-artifacts/34-1-assistant-compte-bancaire.md` (la story qui a posé le contrat).
4. L'issue #1004, **en lecture seule** : `gh issue view 1004 --json number,title,labels,body`. Une seule requête `gh` dans toute la passe.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A0 — chaque finding de la P1 réellement remédié.** Pour M-1 à M-5 et L-1 à L-12 : retrouve dans la fiche l'endroit qui le traite, cite-le, et juge s'il le traite **vraiment** (et non en surface). Relis au code les prémisses neuves de la remédiation : `backup.rs:1058`, `chk_bank_accounts_closed_not_primary` / `card_shape` / `provider` (`crates/kesh-db/test-schema/0001_schema_squash.sql:231-235`), `company_invoice_settings.rs:1584` et `:1682`, `(app)/+layout.ts:42`, `user-manual.tex:2952`. Cherche ce que la remédiation a **créé** : D3-1 (délier un lien de créance en envoyant `null`) est-il cohérent avec la finalisation et avec le refus serveur ? D3-2 et AC-4 (b) tiennent-ils sous Svelte 5 ? AC-5 : le message est-il clair, sans jargon, et dit-il ce qui a échoué et pourquoi l'étape attend ? Une incohérence entre AC, tâches, mutations, captures et Notes de version est un finding.

- **A1 — le serveur est-il juste ?** `crates/kesh-api/src/routes/onboarding.rs:600-746`, `crates/kesh-db/src/repositories/bank_accounts.rs:285-470` :
  `PrimaryDetails`, `is_no_op_change`, l'`UPDATE` dynamique, l'`INSERT`. La fiche a-t-elle raison sur chaque colonne du tableau de l'Étape 0
  (écrasée / non) ? Compare à `columns!()` (`:52-56`) et au schéma (`crates/kesh-db/test-schema/`) : une colonne écrite par un trigger ou
  une contrainte ? `qr_iban` : absent et `null` valent-ils bien « effacé » ?
- **A2 — l'écran.** `frontend/src/routes/onboarding/+page.svelte:229-303`, `:474-505`, `:781-875`, `onboarding.api.ts:63-87`,
  `onboarding.svelte.ts:134-142`. Les numéros de ligne cités sont-ils justes à `4d540a6ea` ? Le cas « lien du compte existant hors de la liste »
  est-il bien traité par l'omission de la clé (D3-1) ? `<select bind:value>` convertit-il en `null` ? L'état `bankLedgerTouched` joue-t-il
  mal avec la valeur chargée ?
- **A3 — l'atteignabilité (Étape 0, point 4).** La fiche dit que l'étape 6 n'a pas de retour en arrière et que le cas exige un compte créé hors
  de l'assistant. Vérifie : `lock_state_at_step`, `reset_demo` (`crates/kesh-seed/src/lib.rs`), la restauration (`crates/kesh-db/src/backup.rs`
  autour de `:1028`), toute route qui baisse `step_completed`. Y a-t-il un chemin **réel** que la fiche ignore (rechargement de page à l'étape 6
  après un envoi dont la réponse s'est perdue : l'état est-il alors 7 ou 6 ?) ? Un constat démenti ne s'écrit pas comme fait.
- **A4 — la lecture du compte existant.** `GET /api/v1/companies/current` (`lib.rs:1322`, `routes/companies.rs:650-665`, `BankAccountJson`
  `:607-629`) : accessible à l'étape 6 pour l'Admin de l'assistant ? Rend-elle l'IBAN entier ? La divergence `list_by_company` (exclut archivés
  **et clos**) contre `SELECT … FOR UPDATE` de l'upsert (`archived = FALSE` seulement) : la fiche la traite-t-elle (D3-3) sans la minimiser ?
  Un principal clos peut-il exister (`closed_on` et `is_primary`) ?
- **A5 — décisions et options écartées.** D1 à D5 : chacune a-t-elle son option écartée, motivée ? D3-2 (bloquer l'étape si la lecture échoue)
  est-il cohérent avec l'AC-6 de la 34-1 (liste du plan en échec : étape utilisable) ? La règle de splitting (CLAUDE.md) : la story touche-t-elle
  plus de cinq modules ? (Compte-les.)
- **A6 — AC constatables.** Chaque AC dit-il un test, une valeur, un fichier ? AC-7 est-il un AC ou une promesse vague ? AC-10 (`git diff` serveur
  vide) est-il vérifiable tel qu'écrit (la forme du pathspec : `crates/*/src/*`, voir la note de mémoire sur le pathspec git muet) ?
- **A7 — tests et mutations.** Les tests Rust T1 prouvent-ils quelque chose alors que le serveur est juste (la fiche dit : non, ils fixent le
  contrat) ? Chaque mutation M1 à M6 rougit-elle le test nommé, par raisonnement sur le code ? Y a-t-il une mutation non couverte (le tri
  « compte principal ordinaire » ; l'omission de `journalAccountId` ; la garde de lecture) ? Le patron « `git checkout --` puis `touch` » est-il
  écrit ?
- **A8 — manuel, PDF compris.** `docs/manual/fr/user-manual.tex:225` et `:2948`, et les mêmes passages en `de`, `en`, `it` s'ils existent.
  Cherche dans les `.tex` **et** dans les PDF, aplatis avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' |
  grep -o -i '.\{150\}présélectionn.\{150\}'`, puis `se modifient ensuite`, `simple proposition`. Une phrase que la story rend fausse ? La phrase
  ajoutée est-elle exacte (renvoyer tel quel ne change rien : vrai pour la version aussi) ?
- **A9 — i18n.** Trois clés neuves × 4 locales : sites réels (`grep -n "onboarding-bank-editable-later" crates/kesh-i18n/locales/*/messages.ftl
  frontend/src`), gardes `i18n-un-repli-par-cle`, `i18n-repli-divergent-actif`, `i18n-keys` (`sitesTotal`), `i18n-libelle-en-dur`, `lint-i18n-ownership`.
  Un site oublié par la fiche ?
- **A10 — recette visuelle et E2E.** Les six captures couvrent-elles chaque état touché (CLAUDE.md § *Recette visuelle*) ? Le plan Playwright
  est-il tenable (amener l'étape 6 par l'API, créer le principal par `POST /api/v1/bank-accounts` — le corps minimal suffit-il, l'étape 6 laisse-t-elle
  passer cette création ?) ? La baseline E2E (KF-029, `docs/testing.md` § *Les échecs attendus*) est-elle respectée ?
- **A11 — sécurité et données réelles.** Préremplir l'IBAN entier : fuite d'écran, de journal, de capture ? Aucune donnée réelle du Project Lead
  dans la fiche ni les captures prescrites. `~/kesh/` jamais touché.
- **A12 — défauts voisins.** Cherche au même endroit un défaut que la fiche n'a pas vu (par exemple le QR-IBAN d'un `PUT`, l'audit
  `bank_account.updated` qui ne trace pas `qr_iban`, l'omission de `description` dans l'instantané). Un défaut voisin se signale « issue à ouvrir »,
  il ne s'intègre pas à la story si sa correction n'est pas mécanique.

## Interdits (commandes nommées)

Lecture seule. N'exécute **aucune** commande qui écrit ou qui change l'état : toute redirection `>` ou `>>` (même vers `/dev/null`), `git add`, `git commit`, `git push`, `git checkout`, `git switch`,
`git reset`, `git rebase`, `git stash`, `git worktree`, `scripts/prepare-release.sh` et tout script mutant, `cargo build`/`test`/`run`/`publish`,
`npm install`/`run`, `sqlx migrate`, `docker`, `docker compose`, `curl`, `python3` et tout interpréteur, `mkdir`, `touch`, `cp`, `mv`, `rm`, `tee`,
toute redirection `>` ou `>>`, **`pkill`, `killall` et `kill` par motif**. Toute commande `gh` qui écrit (`issue create|edit|close|comment`,
`pr`, `api -X`) est interdite ; **une seule** lecture `gh issue view` (une minute d'espacement s'il en faut une seconde, jamais plus de deux).
Autorisés : `Read`, `grep -n`, `grep -nF`, `git log`, `git show`, `git diff`, `git grep`, `ls`, `pdftotext … | …` vers la sortie standard. Si une
commande préalable est refusée, arrête-toi et dis-le.

## Rapport attendu

1. Un « 0 finding » **sans** la liste des axes ne compte pas : la liste des axes **exercés** et **non exercés** (avec la raison) vient en tête.
2. Les findings par sévérité, chacun : prémisse, extrait cité, `fichier:ligne`, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence
   d'un code ou la présence d'un anti-pattern : la preuve par `grep -nF "<chaîne>" <fichier>` collée (règle *Haiku-specific guardrails*, valable pour tous).
3. Le verdict : fiche prête (aucun finding au-dessus de LOW) ou non, et la liste des prémisses de la fiche que tu **démens**.
4. Aucun fichier écrit : le rapport est ta réponse finale.
