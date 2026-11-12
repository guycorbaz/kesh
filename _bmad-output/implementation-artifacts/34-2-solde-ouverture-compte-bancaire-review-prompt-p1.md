# Prompt — revue de code P1 de la Story 34-2 (#533), trois lentilles (B, E, A)

Tu relis la **Story 34-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**,
Epic 34 : **le solde d'ouverture se saisit avec le compte bancaire**, par le mécanisme du mode *Compléter* des soldes de départ
(`opening_complement`), réemployé dans la transaction de la création — et par une route de ligne pour un compte existant. Aucune
migration. Kesh est **en production** (livres réels de Guy) : une écriture fausse se corrige dans des livres tenus.

Worktree `/home/gcorbaz/devel/kesh-e342`, branche `story/34-2-solde-ouverture-compte-bancaire`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e342 diff 133debc2 HEAD` (`133debc2` = base, tête de la 34-1 ; le premier commit de la branche
ne porte que le registre). Langue du rapport : français. Passe **complète** (première passe), contexte frais. Modèle : **Sonnet**
ou **Opus** (jamais Haiku pour une première passe).

Une lentille par agent :

- **B — Blind Hunter** : lis le diff **avant** toute autre source ; cherche les défauts du code tel qu'il est écrit.
- **E — Edge Case Hunter** : parcours chaque branche et chaque borne (corps absent / `null` / `{}` / montant vide, `"0"`, négatif,
  cinq décimales, borne de `DECIMAL(19,4)` ; `side` absent ou inconnu ; compte lié absent, archivé, non imputable, de créance,
  compte de report lui-même ; aucun exercice, premier exercice clos, jour verrouillé ; société avec ou sans écriture ; création
  principale au-dessus d'un principal ; compte clos, archivé, d'une autre société ; rejeu sur interblocage).
- **A — Acceptance Auditor** : AC-1 à AC-10 de la fiche, une par une, contre le code, les tests et les manuels.

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e342-review-p1-B.md`, `…-E.md`, `…-A.md` (le **seul** fichier que tu écris).

## À lire

- La fiche `_bmad-output/implementation-artifacts/34-2-solde-ouverture-compte-bancaire.md` (AC, tâches, mutations M1 à M9,
  *Ce qui ne se traite pas*, **Change Log** et **Dev Agent Record**) ; le registre `e34-choix-autonomes-e341.md` (C-e341-6 à 12,
  38, et C-e341-44 à 50, écrits au développement) ; le fichier d'epic `_bmad-output/planning-artifacts/epic-34-reconciliation-comptes-bancaires.md` ;
  les rapports de validation `/home/gcorbaz/devel/kesh-gate-logs/e34-validate-p*-e341.md` (la fiche a été validée dans le
  groupe e341 ; la clé de travail e342 ne nomme que le worktree et les journaux).
- Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux (lecture seule) sous
  `/home/gcorbaz/devel/kesh-gate-logs/e342/` : `rouges.log`, `rouges-api.log`, `rouges-vitest.log`, `mut/M*.log` (et
  `mut-run-e342.sh`, à **lire**, jamais à exécuter), `gates-*.txt`, `gate-*.log`, `front-*.log`, `vitest-*.log`, `e2e-r*.log`,
  `captures-e342.mjs` (à **lire**), `capture-*.log`.
- **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e342/captures/34-2-01-formulaire-avec-solde.png`,
  `34-2-02-avis-de-succes-et-solde.png`, `34-2-03-refus-compte-mouvemente.png`, `34-2-04-dialogue-de-ligne-dette.png`,
  `34-2-05-dialogue-de-ligne-succes.png`. Regarde-les si ton outil lit les images : le champ n'apparaît-il qu'avec un compte
  lié, la phrase de la conséquence est-elle lisible **avant** le geste, le motif du refus visible, le solde dans la liste ?

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

1. **Les manuels, `.tex` et PDF, contre le code.** `docs/manual/fr/user-manual.tex` : § *Comptes bancaires* (puce du solde,
   § *Le solde d'ouverture d'un compte*, son avertissement, *Un compte existant*, la note) et § *Reprise de comptabilité —
   soldes de départ* (l'avertissement neuf, **avant** l'énumération) ; `docs/manual/fr/admin-manual.tex` (« 139 des 146 routes
   à verbe mutant », « 139 + 4 + 3 = 146 ») ; `docs/api-external.md` (note ¹¹, ligne du tableau, paragraphe des
   interblocages). Chaque phrase est-elle vraie au code (date, sens, atomicité, « l'ancien compte principal reste principal »,
   « refusé si déjà mouvementé — en particulier une seconde fois », codes et ordre des refus) ? Les PDF
   `docs/manual/fr/user-manual.pdf` et `admin-manual.pdf` **sont régénérés** dans la story (l'AC-9 l'exige) : contrôle-les
   aplatis, l'apostrophe du PDF étant `’` (`pdftotext … | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oF "Le solde d’ouverture d’un compte"`,
   `"139 des 146 routes"`).
2. **Un seul mécanisme (AC-1).** `create_opening_complement_in_tx` (`opening_complement.rs`) : l'enveloppe garde sa signature et
   fixe `Screen` ; `SingleBankLedger` distingue « aucun exercice » (`NoOpenFiscalYear`) de « aucune écriture » (lu seulement pour
   `Screen`). Aucun `decide_date`, `check_lines` ni calcul de contrepartie recopié. `tests/opening_complement_repository.rs` et
   `tests/opening_balances_e2e.rs` sont-ils **inchangés** au diff (`git diff 133debc2 HEAD --stat -- <ces fichiers>`) ?
3. **L'instantané et l'ordre des verrous (AC-10, T2).** Dans `create_once`, que précède l'appel du complément ? Seulement des
   lectures verrouillantes ? `claim_accounts_in_share_mode` (lis-le), `refuse_if_ledger_is_claim_account` sur le chemin
   nominal. Le test `ecriture_croisee_pendant_la_creation_est_vue` (montage : ligne ajoutée à une écriture existante, création
   principale au-dessus d'un principal) tue-t-il vraiment M8 — lis `mut/M8.log` — et son raisonnement (une création d'écriture
   serait sérialisée par la sentinelle, via le partagé de clé étrangère sur `companies`) est-il juste ? Route de ligne :
   `companies` (X) → `bank_accounts` (S, `find_by_id_locked_shared_in_tx`) → complément. Un interblocage neuf, non rejoué ?
4. **Atomicité et refus (AC-5).** Sur refus du complément : `conclure` fait-il le `rollback` ? La démotion, l'`INSERT`, les
   traces sont-ils tous après le complément ? Le refus sort-il tel quel (`AppError::Database(DbError::OpeningComplementRefused)`,
   `errors.rs`) — y compris `NoOpenFiscalYear` quand aucun exercice n'existe (message « … ne couvre la date du jour »,
   gardé par la fiche) ?
5. **La forme (AC-2) et les choix C-e341-44, 45.** `parse_opening_balance` : `{}`, `{ "amount": "" }`, `"0"`, `"0.0000"` sans
   `side` → aucun solde ; montant non nul sans `side` → `400` ; `journalAccountId` absent → `400` **avant** toute lecture de la
   base. Le message propre au champ garde-t-il le code `OPENING_COMPLEMENT_INVALID_AMOUNT` ? La réponse sans solde est-elle
   octet pour octet celle d'avant (`#[serde(flatten)]`, `skip_serializing_if`) ? La trace `bank_account.created` d'avant reste-t-elle
   inchangée sans solde ?
6. **Le registre des routes (AC-8).** `audit_route_registry.rs` : la création reclassée `Rejouee`, la route neuve `Traced,
   Rejouee` ; chaque compteur recompté depuis la table (146, 139, 149, 27, 119 ; point (vi) à « vingt-trois ») — recompte-les
   toi-même (`git grep -c`). La route neuve est-elle bien dans le bloc Comptable+ de `lib.rs`, ouverte aux clés `read-write` ?
7. **L'écran (AC-7).** `routes/(app)/bank-accounts/+page.svelte` : `$effect` + `untrack` de la présélection (un rechargement
   défait-il un choix fait à la main ? un `openOpeningBalance` est-il défait par l'effet ?), le snippet `openingFields` partagé,
   l'envoi seulement avec compte lié **et** montant, l'avis par `openingEntryNotice` (numéro, date suisse), le refus dans
   `form-error` / `opening-error`, l'action de ligne seulement sur un compte ouvert **et** lié. `openingAmountPayload`
   (`format.ts`) : la virgule ne remplace que la **première** occurrence — « 1,234,50 » ? Les clés i18n (quatre locales, mêmes
   clés, textes cohérents avec les replis Svelte).
8. **Les témoins et les mutations.** Chaque AC a-t-il un test qui rougit sans le code (`rouges*.log`) ? Un test peut-il passer
   **à vide** (un `waitFor` sur un état déjà vrai ; `expect(...).toBeNull()` sur un élément jamais rendu) ? M2 et M9 sont jouées
   sous une forme **équivalente** déclarée (`mut/M2.log`, `mut/M9.log`) : la forme tient-elle la promesse de la fiche, et la
   déclaration d'équivalence de M9 est-elle juste ?
9. **Limites voisines.** #1002 à #1006 (étape bancaire de l'assistant, ouvertes par la 34-1) : la story les touche-t-elle ?
   La 34-11a2 (carte de crédit, sens *Dette*) pourra-t-elle s'appuyer sur ce code sans le reprendre ?

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la sortie `grep -nF`/`sed -n` qui l'établit,
l'effet observable, le correctif proposé. Puis la liste des **axes exercés** et des **axes non exercés** (avec la raison) : une
passe qui ne la donne pas ne compte pas. Un « 0 finding » se justifie axe par axe.
