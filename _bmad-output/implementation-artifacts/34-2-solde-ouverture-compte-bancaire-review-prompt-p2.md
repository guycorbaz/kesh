# Prompt — revue de code P2 de la Story 34-2 (#533), passe complète, trois lentilles (B, E, A)

Tu relis la **Story 34-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**,
Epic 34 : **le solde d'ouverture se saisit avec le compte bancaire**, par le mécanisme du mode *Compléter* des soldes de départ
(`opening_complement`), réemployé dans la transaction de la création — et par une route de ligne pour un compte existant. Aucune
migration. Kesh est **en production** (livres réels de Guy) : une écriture fausse se corrige dans des livres tenus.

C'est la **passe 2**, **complète**, après la remédiation de la revue P1 (0 CRITICAL, 0 HIGH, 2 MEDIUM, 19 LOW ; rapports
`/home/gcorbaz/devel/kesh-gate-logs/e342-review-p1-{B,E,A}.md`). Modèle : **Sonnet**. Contexte frais. Langue : français.

Worktree `/home/gcorbaz/devel/kesh-e342`, branche `story/34-2-solde-ouverture-compte-bancaire`. Deux diffs, **aplatis** :

- la story entière : `git -C /home/gcorbaz/devel/kesh-e342 diff 133debc2 HEAD` (`133debc2` = base) ;
- la remédiation seule : `git -C /home/gcorbaz/devel/kesh-e342 diff dbdb807c HEAD` (`dbdb807c` = fin du développement). **La sévérité
  se déplace vers ce qu'on vient d'écrire** (CLAUDE.md, § *La passe ciblée*) : lis ce diff-là avec le plus de soin.

Une lentille par agent :

- **B — Blind Hunter** : lis le diff **avant** toute autre source ; cherche les défauts du code tel qu'il est écrit.
- **E — Edge Case Hunter** : chaque branche et chaque borne (corps absent / `null` / `{}` / montant vide, `"0"`, négatif, cinq décimales,
  borne de `DECIMAL(19,4)` ; `side` absent ou inconnu ; nom de banque de 255 et 256 caractères, multioctets ; « 1,234 », « 1,23 »,
  « 1'000,000 », « 1.234,5 » à l'écran ; compte lié absent, archivé, non imputable, de créance, compte de report lui-même ; aucun
  exercice, premier exercice clos, jour verrouillé ; société avec ou sans écriture ; création principale au-dessus d'un principal ;
  compte clos, archivé, d'une autre société, déjà mouvementé ; échec après le complément ; rejeu sur interblocage).
- **A — Acceptance Auditor** : AC-1 à AC-10 de la fiche, une par une, contre le code, les tests et les manuels ; puis **chaque finding
  de la P1**, un par un : remédié (ligne lue), écarté (motif écrit) ou renvoyé à une issue (#1021, #1022).

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e342-review-p2-B.md`, `…-E.md`, `…-A.md` (le **seul** fichier que tu écris).

## À lire

- La fiche `_bmad-output/implementation-artifacts/34-2-solde-ouverture-compte-bancaire.md` (AC, AC-4 annotée, tâches, mutations,
  **Change Log** — l'entrée « remédiation de la revue P1 » — et **Dev Agent Record**) ; le registre `e34-choix-autonomes-e341.md`
  (C-e341-6 à 12, 38, 44 à 55 ; C-e341-46 remplacé) ; le fichier d'epic.
- Les journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e342/` : `rouges.log` (section « Remédiation »),
  `rem1-rouges-*.log`, `mut/resume.txt`, `mut/M*.log` (et `mut/p0/`, les journaux du développement), `mut-run-e342.sh` (à **lire**,
  jamais à exécuter), `rem1-gates-*.txt`, `rem1-gate-*.log`, `rem1-front-*.log`, `e2e-r3.txt`, `e2e-r3.log`, `backend-e2e-r3.log`,
  `captures-e342.mjs`, `capture-r3.log`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le.
- **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e342/captures/34-2-01-formulaire-avec-solde.png` (aperçu du
  montant), `-02-avis-de-succes-et-solde.png` (montant dans l'avis), `-03-refus-compte-mouvemente.png`,
  `-04-dialogue-de-ligne-dette.png` (le dialogue nomme le compte), `-05-dialogue-de-ligne-succes.png`,
  `-06-montant-ambigu-refuse.png`.

⛔ Ne lis rien sous `~/kesh/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`…).
- **Rien qui compile ou exécute** : ni `cargo`, `npm`/`npx`/`node`, `python3`, `perl`, `bash -c`, `make`, `latexmk`, `docker`,
  `mariadb`, `curl`, `playwright`, `vitest`, `claude` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
  `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`.
- **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`),
  `wc`, `sort`, `uniq`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` (sous `timeout 60`).
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Les manuels, `.tex` et PDF aplati, contre le code.** `docs/manual/fr/user-manual.tex` § *Le solde d'ouverture d'un compte*
   (avertissement au conditionnel, aperçu et refus d'un montant ambigu, jour UTC et #1022, liste des refus, *Un compte existant* :
   action masquée, dialogue qui nomme le compte, refus du compte de créance, trace d'audit) et § *Soldes de départ* ;
   `docs/manual/fr/user-manual.pdf` aplati (apostrophe `’`) ; `admin-manual.tex` (« 139 des 146 routes ») ; `docs/api-external.md`
   note ¹¹ (codes de la route de ligne, `400 LATER_FISCAL_YEAR_CLOSED`, borne du nom de banque, trace neuve). Chaque phrase est-elle
   vraie au code ?
2. **E-1 et M9.** Le témoin `un_echec_apres_le_complement_n_ecrit_rien` (déclencheur `SIGNAL` sur `bank_accounts`, création
   principale au-dessus d'un principal) : prouve-t-il ce qu'il dit (lis `mut/M9.log` : `(true, false, false, false)`) ? Le constat
   « aucun compte » est-il discriminant sous M9 (il reste vrai : pourquoi) ? Un autre échec après le complément (trace, relecture,
   1205 sur la démotion) est-il couvert par le même raisonnement ? La borne du nom de banque (`refuse_too_long_bank_name`) est-elle
   au bon endroit dans les **trois** appelants (création, `PUT`, assistant), en caractères et non en octets ? Le `PATCH` n'en a pas
   besoin (il ne touche pas le nom) : vérifie.
3. **La route de ligne** (`enter_opening_balance_once`) : ordre des refus (`404` → `409` clos → `412` → `400` créance → complément),
   lectures verrouillantes avant le complément (sentinelle, ligne en partagé, réglages en partagé), ordre des verrous comparé à la
   création, trace `bank_account.opening_balance_entered` dans la transaction, action au vocabulaire (`audit_labels.rs`, quatre
   locales). La garde `refuse_if_ledger_is_claim_account` lit le numéro du compte par une lecture **ordinaire** sur le chemin du refus
   seulement : est-ce sans effet sur l'instantané du chemin nominal ?
4. **L'écran** : `openingAmountAmbiguous` (`/^\d+,\d{3}$/` après retrait des séparateurs) — faux positifs, faux négatifs (« 1,234.5 »,
   « ,123 », « 1 234,567 », « 0,500 ») ; `openingAmountPreview` (ce qu'il montre est-il **exactement** ce que le serveur écrira, ou
   arrondi au centime alors que le serveur garde quatre décimales ?) ; l'avis qui rappelle le montant (`formatChfBalance` d'un
   montant invalide est-il atteignable ?) ; l'action masquée par `lastTransactionDate` (un compte lié **partagé** par deux comptes
   bancaires ?) ; le titre du dialogue ; les quatre locales des clés neuves et modifiées contre les replis Svelte.
5. **Les compteurs** : `audit_route_registry.rs` (146, 139, 149, 27, 119, « vingt-trois ») inchangés par la remédiation ? `sitesTotal`
   de `i18n-keys.test.ts` (2944, +2 dans la remédiation) recompté ?
6. **Les témoins et les mutations** : M1 à M11 rejouées après la remédiation (`mut/resume.txt`) ; un test peut-il passer **à vide** ?
   La comparaison du corps entier (B-8) compare-t-elle bien deux objets indépendants (la réponse HTTP et la sérialisation du compte relu
   en base) ?
7. **L'E2E** : le Dev Agent Record nomme-t-il chaque échec du run de la remédiation contre `docs/testing.md` (sept KF-029) et #1021
   (KF-059, verrou tenu après `export-global.spec.ts`), avec les lignes du journal du serveur ?
8. **Limites voisines** : #1002 à #1006 (assistant), #1021, #1022 — la story les touche-t-elle ? Le nom de banque de l'assistant est
   désormais borné : `onboarding_bank_account_e2e.rs` en dit-il quelque chose ?

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la sortie `grep -nF`/`sed -n` qui l'établit,
l'effet observable, le correctif proposé. Puis la liste des **axes exercés** et des **axes non exercés** (avec la raison) : une
passe qui ne la donne pas ne compte pas. Un « 0 finding » se justifie axe par axe.
