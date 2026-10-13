# Story 28.8 : Cinq petits correctifs indépendants — un test daté du matin, une fonction morte, trois documents

## Status

**review** — développée le 2026-10-14 (`bmad-dev-story`, Sonnet 5.5, autonomie), cinq commits de code/doc `ebb9bc81`..`7d69e5ab` puis le relevé et le Dev Agent Record. Historique : fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `6de5a7ff`
(`origin/main`, 28-7 fusionnée), branche `story/28-8`, worktree `/home/gcorbaz/devel/kesh-28-8`. Validation close en P1
(Sonnet, une lentille R+F : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW — tous remédiés, cf. Change Log). Choix consignés au registre `epic-28-choix-autonomes.md`, **C-28-8-1 à C-28-8-8**.

9 AC, T0-T8. Cinq issues, chacune **entièrement** réglée ou sortie du périmètre : [#421] (KF-051), [#561], [#595],
[#612], [#615].

⛔ **Consigne pour l'orchestrateur — la PR, et elle seule, porte `closes #421 #561 #595 #612 #615`** (titre ou corps,
le dépôt merge en squash). Les commits de la branche portent **`refs #N`**, jamais `closes`/`fixes` : un mot-clé de
fermeture dans un message de commit fermerait l'issue dès ce commit poussé ou cité. Après merge, vérifier
`gh issue view <N> --json state` pour les cinq.

## Story

**As a** Guy, qui veut que chaque release ait moins d'issues ouvertes que la précédente,
**I want** fermer cinq défauts déjà diagnostiqués, chacun à son minimum,
**so that** le décompte baisse sans risque : un test qui échouait le matin, une fonction morte au doc-comment
inexact, deux listes de refus incomplètes, une brochure qui range au backlog une fonction livrée, un gabarit `.env`
du manuel qui contredit `.env.example`.

Sources : le tri en lecture seule `/home/gcorbaz/devel/kesh-gate-logs/triage-issues-28.md` (hors dépôt ; lignes
50, 82-83, 88, 108, 148-158) et les cinq issues, lues le 2026-10-13 (`gh issue view N --comments`).

**Effet sur le décompte des bugs** (fichier d'epic, § *Le décompte*) : #421 porte `known-failure`, #595 porte
`bug` → **−2**. #561 (`technical-debt`), #612 et #615 (`documentation`) ne sont pas comptés.

## Le constat, au sol (`6de5a7ff`)

| Issue | Constat | Preuve relevée |
|---|---|---|
| [#421] KF-051 (ex-KF-045) | Le helper `recordManualReminderViaApi` poste `sentAt: \`${today}T12:00:00\`` ; la route refuse une date d'envoi future en la comparant à l'heure **UTC** : avant 12:00 UTC (14:00 en Suisse l'été), midi est futur → `422`, et deux tests rougissent | `frontend/tests/e2e/invoices.spec.ts:262-265` ; `crates/kesh-api/src/routes/dunning_reminders.rs:261` (`if body.sent_at > Utc::now().naive_utc()`) ; tests `:415` « historique des rappels affiché (canal manuel, ligne visible) » et `:439` « axe-core sans violations sur la fiche (badge suspendu + historique) » ; aucune autre garde de date dans `record_manual_reminder` (`:248-335`) |
| [#421] (doc) | `docs/testing.md` nomme encore ces deux échecs **KF-045** (renumérotée KF-051 le 2026-09-10) et `sidebar-navigation.spec.ts:75` **KF-046** (renumérotée KF-052, #424) | `docs/testing.md:331`, `:360-362`, `:364-372`, `:374`, `:380-383`, `:386-387`, `:390-394` ; `CLAUDE.md:706` (la renumérotation) ; titre de #424 : `[KF-052] sidebar-navigation.spec.ts:75 …` |
| [#561] | `accounts::delete_all_by_company` n'a **aucun appelant**, tests compris ; son doc-comment dit « (tests) », inexact | `crates/kesh-db/src/repositories/accounts.rs:1134-1159` ; `git grep -n "accounts::delete_all"` → seule une fiche d'archive `_bmad-output` ; les appels non qualifiés `delete_all_by_company(` du dépôt sont tous dans `journal_entries.rs` (sa propre fonction, `:2035`) ou `journal_entry_reversal_e2e.rs:905` (`journal_entries::`) ; les imports `use kesh_db::repositories::{accounts, …}` n'importent que le module |
| [#595] | Les deux listes de refus d'`api-external.md` qui passent par la file commune `settlement_entry_cancel_blocker` omettent `MATCHED_BANK_TRANSACTION` | `docs/api-external.md:526` (annuler le règlement d'une facture fournisseur) et `:534` (annuler une facture fournisseur) — l'issue cite `:426` et « la liste voisine `:434` », relevés sur `056997b0` : ce sont ces deux listes (`git show 056997b0:docs/api-external.md \| sed -n 420,436p`) |
| [#612] | La brochure range au backlog « Report automatique des soldes de bilan à la clôture d'exercice », livré en v0.8.0 | `docs/manual/fr/marketing-brochure.tex:425-430` ; `CHANGELOG.md:578` (section `[0.8.0]` : « Report à-nouveau automatique … sans aucune écriture de clôture ni de report à passer à la main ») ; `README.md:218` ; `user-manual.tex:737` ; PDF : `pdftotext … \| tr '\n' ' ' \| tr -s ' '` rend « Backlog (Epic 13 à 15) • Report automatique des soldes de bilan à la clôture d'exercice. » |
| [#615] | Le gabarit `.env` du manuel écrit `MARIADB_ROOT_PASSWORD=<sortie de openssl rand -hex 32>` et `MARIADB_PASSWORD=<autre sortie …>` **décommentés, entre chevrons** ; `.env.example` les laisse **commentés et vides** | `docs/manual/fr/admin-manual.tex:258-263` ; `.env.example:354-372` (`#MARIADB_ROOT_PASSWORD=`, `#MARIADB_PASSWORD=`, consigne « INSTALLATION NEUVE : … décommenter la ligne et y écrire la valeur ») ; PDF : `MARIADB_ROOT_PASSWORD=<sortie de openssl rand -hex 32>` |

### La chaîne du refus `MATCHED_BANK_TRANSACTION` (#595), relue au code

- **Annuler le règlement d'une facture fournisseur** (`POST /supplier-invoices/{id}/settlement/cancel`) :
  `supplier_invoices::cancel_settlement_in_tx` (`repositories/supplier_invoices.rs:1292`) ne refuse **lui-même**
  que `SupplierInvoiceNotPaid`, `FiscalYearClosed` et `DocumentLetteringInClosedPeriods` (`:1343-1355`) ; il laisse
  les autres rangs « au socle, qui les refuse avec son erreur canonique » : `journal_entries::reverse_owned_in_tx`
  (`:1384`, autorité `SupplierSettlement`), dont l'étape (1) (`journal_entries.rs:2664-2677`) oppose le premier motif
  de `reversal_blockers` qui n'est ni `AccountArchived` ni exempté — et `exempts` (`:2534-2549`) n'exempte, pour
  cette autorité, que `OwnedBySupplierInvoice`. Un règlement rapproché lève donc
  `DbError::EntryNotReversable { blocker: MatchedBankTransaction, document_id: <transaction> }`, rendu en **`409
  MATCHED_BANK_TRANSACTION`** avec `details.documentId` (`errors.rs:1303-1305`, `entry_document_refusal_response`
  `:1327-1339`). Rang : après le lettrage figé (rang 2 bis), avant `ACCOUNT_ARCHIVED` (rang 4) —
  `settlement_cancellation.rs:130-171`.
- **Annuler une facture fournisseur** (`POST /supplier-invoices/{id}/cancel`) : même file, évaluée sur l'écriture
  d'**achat** (`supplier_invoice_cancel_blocker`, `supplier_invoices.rs:995-1036`) ; le geste ne refuse lui-même
  que `SupplierInvoiceCancelled`, `FiscalYearClosed`, `DocumentLetteringInClosedPeriods` et
  `SupplierInvoiceInPaymentBatch` (`:1102-1115`, contre-passation `:1139`) ; le rapprochement est opposé par le socle (autorité `SupplierPurchase`, même
  `exempts`). Rang : après le lettrage figé, avant `ACCOUNT_ARCHIVED` — et donc avant
  `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`, que la fonction de lecture n'atteint qu'après la file.
- ⚠️ **Le refus est défensif** (C-28-8-5) : les cinq sites qui posent `bank_transactions.matched_entry_id`
  (`routes/reconciliation.rs:2123` `accept_one_invoice`, `:2622` `accept_one_split`, `:3074` `accept_one_rule`,
  `:3901` `post_manual_once`, `:4364` `post_split_once`) rapprochent tous une écriture **qu'ils viennent de créer**
  (`journal_entries::create_in_tx`, ou `invoice_settlements::create_in_tx` pour `accept_one_invoice`, qui règle une
  facture **client**), jamais une écriture existante ; aucun ne crée l'écriture de règlement ni
  d'achat d'une facture fournisseur. Aucun geste de Kesh ne rapproche donc aujourd'hui ces écritures d'une
  transaction : la documentation liste le refus parce que la route le **peut** rendre, et le dit défensif. Seul
  l'import d'installation (`backup.rs`) recopie la colonne telle quelle — il restitue ce qu'une instance Kesh a
  écrit, donc rien de plus.
- ⚠️ Sur les deux routes, `bank_accounts::ensure_open_in_tx` (28-4a, `409 BANK_ACCOUNT_CLOSED`) passe **avant**
  tous ces motifs (`supplier_invoices.rs:1340` pour le règlement) ; son inscription aux listes revient à la 28-4b.

## Critères d'acceptation

**AC1 — #421 : le rappel manuel de l'E2E est daté du passé proche (C-28-8-2).**

- `frontend/tests/e2e/invoices.spec.ts`, helper `recordManualReminderViaApi` : `sentAt` vaut
  `new Date(Date.now() - 3_600_000).toISOString().slice(0, 19)` (une heure avant l'appel, en UTC, sans fuseau ni
  millisecondes — un `NaiveDateTime`, ce que `#249` exige) ; la variable `today` disparaît ; le commentaire `#249`
  reste et dit pourquoi une heure avant (« la route refuse une date future, comparée à l'heure UTC — #421 »).
- **Jumeaux** : `git grep -nE "T[0-9]{2}:[0-9]{2}:[0-9]{2}" frontend/tests/e2e` et `git grep -n "sentAt"
  frontend/tests/e2e` ne rendent, hors fixtures CAMT, que le site corrigé (relevé au sol : seul `invoices.spec.ts:265`
  code une heure ; seul ce helper poste `reminders/manual`). Le résultat se copie au Dev Agent Record. Le jumeau
  **côté produit** — la date par défaut d'un rappel saisi à l'écran (`reminder-sent-at.ts`) — est déjà corrigé
  (#259, `CHANGELOG.md:602`) : à noter au Dev Agent Record, rien à changer.
- **Preuve** : les deux tests passent rejoués seuls (`npx playwright test invoices.spec.ts -g "historique des
  rappels|badge suspendu \+ historique"`) ; **mutation M1** (`Date.now() + 3_600_000`, date future) les fait rougir
  tous deux en `manual reminder failed: 422` — elle rend le défaut d'origine à toute heure. L'heure UTC du run est
  notée au Dev Agent Record ; un run avant 12:00 UTC vaut preuve directe, un run après n'en est pas une — d'où M1.

**AC2 — #421 : la liste des échecs attendus de `docs/testing.md` est mise à jour, sans réécrire ses relevés datés
(C-28-8-2, C-28-8-3).**

Dans la § *« Les échecs attendus »* :

- **le chapeau** (`:330-333`) ne dit plus que le nombre dépend de l'heure : il donne le compte attendu (cf. puce
  formule) et renvoie à la note historique ;
- **le tableau** (`:349-362`) perd les deux lignes `invoices.spec.ts:415` et `:439` ; la ligne
  `sidebar-navigation.spec.ts:75` y nomme **KF-052 (#424)** ; une note datée sous le tableau dit le retrait
  **et** la nature du tableau (« 2026-10-13, Story 28-8 : malgré son intitulé, ce tableau est la liste **vivante** des
  échecs attendus — il a reçu des lignes après le 2026-08-26 ; les deux lignes KF-051 (#421, ex-KF-045) en sont
  retirées, le rappel de l'E2E étant daté d'une heure avant l'appel ») ;
- **le paragraphe ⏰** (`:364-372`) devient une note historique courte, au passé : la cause (midi comparé à l'heure
  UTC), la correction (Story 28-8), et la leçon, conservée — « un test qui dépend de l'heure ne rougit que pour la
  moitié des gens qui le lancent » ;
- **les paragraphes de la KF-046** (`:374-384`) la nomment **KF-052** ;
- **la formule** (`:386-387`) devient `7 + (1 si la KF-052 se déclenche) + 0 à 2 de pollution`, soit **7 à 10**
  — la borne basse passe à 7 parce qu'un run sans aucune pollution est observé (registre de sprint, 28-4a :
  « E2E 273 + 7 KF-029 ») ; C-28-8-3 ;
- **le tableau historique du 2026-08-30** (`:388-394`, colonne `KF-045`, cellule « plus **KF-046** ») **reste tel
  quel** — c'est un relevé daté (consigne 15 ; § *Recompter n'est pas réécrire l'historique* du `CLAUDE.md`) — et
  reçoit une annotation datée : « *KF-045 et KF-046 ont été renumérotées KF-051 et KF-052 le 2026-09-10 ; la
  première est corrigée par la Story 28-8.* » ;
- **le relevé de la story** : le run E2E complet de T7 s'y ajoute en une ligne datée (passés / échoués, liste
  nominative), en tête du tableau des relevés ou sous la formule.

Preuve : `grep -nE "KF-045|KF-046" docs/testing.md` ne rend plus que le tableau historique du 2026-08-30 et son
annotation ; `grep -nE "le matin|matinal|10 le matin|8 à 12" docs/testing.md` ne rend que la note historique
**et** le paragraphe KF-052 conservé (« a passé seule le matin du 2026-08-30 », `:376` — un fait daté, juste) ;
sortie copiée au Dev Agent Record. Aucun autre fichier versionné ne porte le compte « 8 à 12 » ni « 10 le matin »
(`git grep -nE "8 à 12|10 le matin" -- ':(exclude)_bmad-output'` au sol : `docs/testing.md:331`, `:387` seulement).

**AC3 — #561 : `accounts::delete_all_by_company` est supprimée (C-28-8-4).**

- La fonction et son doc-comment (`accounts.rs:1134-1159`, accolade fermante comprise) disparaissent ; rien d'autre ne change dans le fichier,
  hormis un import devenu inutilisé s'il y en a un (clippy le dit).
- **Preuve d'absence d'appelant, avant suppression** : `git grep -nw "delete_all_by_company"` copié au Dev Agent
  Record, chaque ligne classée (journal_entries, commentaire, archive `_bmad-output`) ; **après** :
  `cargo build --workspace --all-targets` et `cargo clippy --workspace --all-targets -- -D warnings` verts — la
  compilation de toutes les cibles, tests compris, est la preuve qu'aucun appelant ne subsistait.
- Pas de mutation : une suppression ne se mute pas ; la contre-épreuve est la compilation.
- Les doc-comments qui citent **`journal_entries::delete_all_by_company`** (`letterings.rs:30`,
  `letterings_lexical.rs:23`, `journal_entries.rs:74`, `invoices.rs:4493`, `docs/testing.md:36`, `CLAUDE.md:511`,
  `docs/migrations-idempotence-audit.md:86`) restent vrais et ne bougent pas.

**AC4 — #595 : les deux listes de refus nomment `MATCHED_BANK_TRANSACTION`, à leur rang (C-28-8-5).**

- `docs/api-external.md:526` (*Annuler le règlement d'une facture fournisseur*) : entre
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS (…)` et `ACCOUNT_ARCHIVED`, l'entrée « `MATCHED_BANK_TRANSACTION` (`409`,
  `details.documentId` = la transaction ; refus défensif — aucun geste de Kesh ne rapproche aujourd'hui l'écriture
  de règlement d'une facture fournisseur d'une transaction bancaire) ».
- `docs/api-external.md:534` (*Annuler une facture fournisseur*) : entre `LETTERING_ALL_LINES_IN_CLOSED_PERIODS
  (…)` et `ACCOUNT_ARCHIVED`, l'entrée « `MATCHED_BANK_TRANSACTION` (`409`, `details.documentId` = la transaction ;
  l'écriture d'**achat** est rapprochée — refus défensif, au même titre) ».
- Rien d'autre ne change dans ces deux paragraphes ; ⚠️ la 28-4b y ajoutera `409 BANK_ACCOUNT_CLOSED` (fiche 28-4,
  AC16) : conflit textuel possible au rebase, résolu par **union** des deux insertions.
- Preuve : la chaîne du § *La chaîne du refus*, relue et citée au Dev Agent Record (`sed -n` des lignes nommées) ;
  `grep -c "MATCHED_BANK_TRANSACTION" docs/api-external.md` passe de **6** à **8** (recompté au sol : `:262`, `:296`,
  `:438`, `:454`, `:476`, `:570`).

**AC5 — #612 : le report des soldes de bilan quitte le backlog de la brochure et rejoint les fonctions livrées
(C-28-8-6).**

- `marketing-brochure.tex:428` : la puce « Report automatique des soldes de bilan à la clôture d'exercice. » sort
  du bloc *Backlog (Epic 13 à 15)* ; les deux autres puces restent.
- Un bloc livré, au patron des blocs voisins (*TVA Suisse (Epic 11 \& 18, livré)*, *Lettrage et postes ouverts
  (Epic 15, v0.13)*), placé **entre** celui de la TVA (v0.3) et celui du lettrage (v0.13) — ordre des versions : titre
  `\textbf{\color{keshSuccess}\faCheckCircle\ Report à-nouveau automatique (Epic 14, v0.8, livré)}`, une puce :
  « Report automatique des soldes de bilan d'un exercice à l'autre, calculé en temps réel depuis l'origine : aucune
  écriture de report à passer. \emph{(Le bouclement d'exercice --- écritures de clôture, report à nouveau du résultat,
  transitoires --- reste à venir.)} » — la réserve, parce que le bouclement ([#232], ouvert) n'est pas livré et que le CHANGELOG de la
  0.13.0 le dit manquant (`CHANGELOG.md:135`).
- PDF régénéré (`make brochure`, sous le verrou) et contrôlé aplati :
  `pdftotext docs/manual/fr/marketing-brochure.pdf - | tr '\n' ' ' | tr -s ' '` rend le nouveau bloc et ne rend plus
  la puce sous « Backlog » ; sorties copiées au Dev Agent Record.

**AC6 — #615 : le gabarit `.env` du manuel d'administration suit `.env.example` (C-28-8-7).**

- `admin-manual.tex:258-263` : les lignes MariaDB du `lstlisting` deviennent, dans l'ordre de `.env.example`
  (`:369-372`) :

  ```
  # Mots de passe MariaDB : OBLIGATOIRES avec docker-compose.yml, lus à
  # la création de la base seulement. Installation NEUVE : générer chacun
  # par openssl rand -hex 32, décommenter la ligne et y écrire la valeur.
  # Installation EXISTANTE : la valeur avec laquelle la base a été créée.
  #MARIADB_ROOT_PASSWORD=
  MARIADB_DATABASE=kesh
  MARIADB_USER=kesh
  #MARIADB_PASSWORD=
  ```

  (texte des commentaires libre, à ces quatre faits près ; aucune valeur entre chevrons sur ces lignes).
- **La prose qui précède dit quoi y écrire** : la phrase « sur une installation neuve, générez chacun par
  `openssl rand -hex 32` » (`:246`) gagne « , **décommentez** les deux lignes et écrivez-y la valeur » ; et la phrase
  « Décommentée sans valeur, une ligne fait aussi refuser Compose » de `.env.example:366-367` y est reprise. Le
  reste du paragraphe (chevrons acceptés par Compose et MariaDB, avertissement de Kesh pour le seul mot de passe
  applicatif, rien pour root) **reste** : il demeure vrai — une valeur entre chevrons peut encore être écrite à la
  main ou recopiée d'une version antérieure du manuel, et `DATABASE_URL` en porte une (`:267`).
- **Les bases déjà créées avec le gabarit retiré** (validation P1, RF1) : le paragraphe `:248` nomme les chaînes
  littérales des manuels **antérieurs à la 0.13.0** (`<mot de passe fort>`, `<mot de passe utilisateur applicatif>`) ;
  les manuels **0.13.0 à 0.13.2** prescrivaient `<sortie de openssl rand -hex 32>` et `<autre sortie de openssl rand
  -hex 32>` (`git show v0.13.2:docs/manual/fr/admin-manual.tex | sed -n 260,261p`), que la story retire du listing.
  Ces deux chaînes s'ajoutent à l'énumération du paragraphe `:248` (« recopiées d'une version antérieure de ce
  manuel »), avec la même consigne : ne pas les remplacer dans `.env`, changer le mot de passe par le
  § `sec:mariadb-mdp`. Preuve : `grep -nF "autre sortie de openssl rand" docs/manual/fr/admin-manual.tex` ne rend
  plus que ce paragraphe.
- ⚠️ La consigne de la story dit « le gabarit `<…>` est refusé au démarrage (`is_template_placeholder`) » : c'est
  vrai pour `KESH_JWT_SECRET` et `KESH_ADMIN_PASSWORD` seulement (`config.rs:637`, `:670`) ; pour le mot de passe de
  `DATABASE_URL`, `is_template_placeholder` ne fait qu'**avertir** (`config.rs:1458`, `AVERTISSEMENT_MOT_DE_PASSE_GABARIT`)
  et `MARIADB_ROOT_PASSWORD` n'est pas lu par Kesh. Le manuel ne doit **pas** écrire de refus pour les `MARIADB_*`.
- Preuve : `git grep -nE "MARIADB_(ROOT_)?PASSWORD=<"` ne rend plus rien (au sol : `admin-manual.tex:260`, `:261`
  seulement) ; PDF régénéré (`make admin`) et aplati : `#MARIADB_ROOT_PASSWORD=` et `#MARIADB_PASSWORD=` présents,
  `MARIADB_ROOT_PASSWORD=<sortie` absent ; sorties copiées.

**AC7 — CHANGELOG (C-28-8-8).**

Dans la section **unique** `## [0.13.3] — Non publié` (`CHANGELOG.md:11`), rubrique `### Corrigé` existante, une
puce par union avec les entrées des autres stories :

« **Documentation corrigée.** L'API externe (`docs/api-external.md`) liste désormais le refus
`MATCHED_BANK_TRANSACTION` de l'annulation d'un paiement fournisseur et de l'annulation d'une facture fournisseur
([#595](https://github.com/guycorbaz/kesh/issues/595)) ; le manuel d'administration présente les mots de passe MariaDB du fichier `.env` commentés, comme
`.env.example`, et dit quoi y écrire sur une installation neuve ([#615](https://github.com/guycorbaz/kesh/issues/615)) ; la brochure range le report automatique
des soldes de bilan, livré en v0.8.0, parmi les fonctions livrées ([#612](https://github.com/guycorbaz/kesh/issues/612)). »

⚠️ Liens **en ligne** : le CHANGELOG ne porte aucune définition de lien de référence (`grep -c '^\[#[0-9]*\]:'
CHANGELOG.md` → 0) ; un `([#595])` nu s'afficherait en clair.

#421 (test) et #561 (code mort) n'ont pas d'entrée : invisibles pour l'utilisateur (C-28-8-8). Aucune autre
section du CHANGELOG ne change ; aucune section `[0.13.3]` n'est créée en double.

**AC8 — Sites non résolus et propagation (C-28-8-1).**

Pour chaque correctif, le symptôme — la **valeur** — se grepe sur tout le dépôt après le patch (§ *Propagation*),
et le Dev Agent Record copie la sortie. Les sites restés tels quels sont écrits et justifiés :

- #421 : le tableau historique du 2026-08-30 de `docs/testing.md` (relevé daté) ; les fiches et rétrospectives de
  `_bmad-output` qui nomment KF-045 (archives) ;
- #561 : les archives `_bmad-output` (`15-5d-…`, `epic-15-choix-autonomes.md`) qui décrivent la fonction ;
- #595 : aucun autre paragraphe d'`api-external.md` ne liste les refus de ces deux routes (`grep -n
  "SUPPLIER_INVOICE_NOT_PAID\|SUPPLIER_INVOICE_CANCELLED" docs/api-external.md` au sol : `:526`, `:534`) ; le tableau
  des codes d'erreur (§ 10) n'est pas une liste par route ;
- #612 : `README.md:218` et `user-manual.tex:737` disent déjà la livraison ; l'intitulé « Backlog (Epic 13 à 15) »
  n'est pas réaudité (hors #612) ;
- #615 : `admin-manual.tex:2231-2232` (`echo "MARIADB_ROOT_PASSWORD=$NEW_ROOT"`) est la procédure de changement de
  mot de passe, non un gabarit : inchangé.

**AC9 — Gates.**

- **`kesh-db` est touché → gate backend complet** : `scripts/test-fast.sh` (fmt + clippy + nextest, sous
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`) sur la base `kesh_288` **remise à zéro juste avant**
  (drop/create, migrations du worktree, seed) — jamais de `docker restart` de `kesh-mariadb-dev` (consigne 2) ;
  `cargo build --workspace --all-targets` vert.
- **Frontend** (série de la CI) : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run
  build` ; Vitest sous le verrou.
- **Manuels** : `make admin brochure` (dans `docs/manual/`, sous le verrou) ; les deux PDF commités ; contrôles
  aplatis des AC5 et AC6 ; `textes_coherents.rs` et `configuration_transmise.rs`, qui lisent ces `.tex`, sont dans
  le nextest complet.
- **E2E complet au dernier commit de code** (le dernier commit qui touche `invoices.spec.ts` ou `accounts.rs`) — et,
  les `.tex` étant lus par des tests Rust, le gate backend complet se rejoue si un commit de documentation suit le
  dernier nextest. Montage de `docs/testing.md` à la lettre, port **3042**, base **`kesh_e2e_288`** reconstruite,
  sous le verrou. **Attendu** : la liste mise à jour de l'AC2 — les 7 KF-029, la KF-052 éventuelle, 0 à 2 de
  pollution (chacune rejouée seule, verte) — et **ni** `invoices.spec.ts:415` **ni** `:439`, quelle que soit l'heure.
  Tout autre échec est une régression jusqu'à preuve du contraire.
- **PDF et rebase** (validation P1, RF2) : la 28-1, la 28-3 et la 28-4 touchent aussi `admin-manual.tex` (et son
  PDF) ; aucune garde ne lit les PDF. Les deux PDF se **régénèrent après le dernier rebase** sur `origin/main`, sous
  le verrou, et c'est cette version qui se commite ; un conflit sur un PDF ne se résout **jamais** par `--ours` ni
  `--theirs` — on résout le `.tex` par union, puis on régénère. Même union au rebase pour `docs/testing.md`,
  `CHANGELOG.md` et `docs/api-external.md`.
- Le Dev Agent Record ne déclare que ce qui a tourné, avec les chiffres recomptés.

## Tasks / Subtasks

- [x] **T0** — Montage : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-8/target` ; `node_modules` par `cp -al` depuis
  le dépôt principal ; bases `kesh_288` et `kesh_e2e_288` créées (migrations du worktree, seed
  `scripts/seed-dev-db.sql`) ; relire les numéros de ligne de cette fiche si `origin/main` a bougé (rebase).
- [x] **T1** — #421 (AC1) : grep des jumeaux (copié), correction du helper, commentaire ; M1 (mutation) puis
  restauration (`git checkout -- f && touch f`) ; commit `test(28-8): … (refs #421)`.
- [x] **T2** — #561 (AC3) : grep copié et classé ; suppression ; `cargo build --workspace --all-targets` + clippy ;
  commit `refactor(kesh-db): … (refs #561)`.
- [x] **T3** — #421 doc (AC2) : `docs/testing.md` ; commit `docs(testing): … (refs #421)` — ou avec T1.
- [x] **T4** — #595 (AC4) : relire la chaîne au code (sed -n copiés), deux insertions ; commit `docs(api): … (refs #595)`.
- [x] **T5** — #612 (AC5) : brochure, `make brochure`, contrôle aplati ; commit `docs(brochure): … (refs #612)`.
- [x] **T6** — #615 (AC6) : manuel d'administration, `make admin`, contrôle aplati ; commit `docs(admin): … (refs #615)`.
- [x] **T7** — CHANGELOG (AC7) ; propagation (AC8) ; puis gates complets au HEAD (AC9) : test-fast complet sur base
  remise à zéro, frontend, E2E complet ; relevé E2E versé à `docs/testing.md` (AC2, dernière puce) ; commit
  `docs: … (refs #421, refs #595, refs #612, refs #615)`.
- [x] **T8** — Dev Agent Record, File List, Change Log ; statut `review` ; registre de sprint. Rapport : demander à
  l'orchestrateur le push, la PR (`closes #421 #561 #595 #612 #615`) et l'issue de C-28-8-6.

## Dev Notes

### Dérogation à la règle de découpage

La story touche six emplacements : `frontend/tests/e2e`, `docs/testing.md`, `crates/kesh-db`,
`docs/api-external.md`, `docs/manual/fr` (deux `.tex`, deux PDF), `CHANGELOG.md`. **Au sens de la règle** (crates
Rust, packages npm, modules métier de premier niveau), ce sont **trois** modules — `kesh-db`, le frontend (tests E2E
seulement), la documentation — et le critère « scope cross-cutting » du § *Règle de splitting préventif* **n'est pas
atteint** ; le second critère (non-convergence) ne peut l'être qu'en validation. La section est donc une précaution,
non une exception codifiée (validation P1, RF11) : si l'on compte chaque document comme un module, elle tient lieu
de la section `Dérogation règle de splitting` qu'exige l'exception du `CLAUDE.md`, **déclarée au Project Lead par
le rapport de l'orchestrateur**. Dans les deux lectures, la story **n'est pas découpée** :

- **les cinq correctifs sont indépendants** — aucun ne lit, n'appelle ni ne suppose un autre ; aucun ne change une
  règle métier ; un seul touche du code de production (une suppression sans appelant), un seul un test ;
- **chacun est mécanique** et tient en un fichier (deux pour #421 : le test et sa liste d'échecs attendus) ;
- **la revue se fait fichier par fichier**, comme le § prescrit pour une sous-story de rollout, et non en passes
  adversariales globales : chaque correctif se juge contre son issue et son AC, sans modèle mental commun à tenir.

Cinq stories d'un fichier chacune coûteraient cinq cycles de CI complets (§ *Quand sauter* : un commit doc-only
déclenche les trois jobs) pour le même diff. Risque accepté : un défaut d'un correctif retarde la PR des quatre
autres ; remède : sortir le correctif fautif du périmètre (consigne de la story : « chaque issue doit être
entièrement réglée, sinon elle sort du périmètre ») plutôt que retenir la PR. C-28-8-1.

### Propagation — le symptôme à greper après chaque patch

```sh
git grep -nE "T12:00:00|\\\$\{today\}T" frontend/tests/e2e                       # #421
git grep -nE "KF-045|KF-046" -- ':(exclude)_bmad-output'                          # #421 (doc)
git grep -nE "8 à 12|10 le matin|matinal" -- ':(exclude)_bmad-output'             # #421 (doc)
git grep -nw "delete_all_by_company"                                              # #561
grep -n "SUPPLIER_INVOICE_NOT_PAID\|SUPPLIER_INVOICE_CANCELLED\|MATCHED_BANK_TRANSACTION" docs/api-external.md  # #595
git grep -n "Report automatique des soldes" -- docs website README.md             # #612
git grep -nE "MARIADB_(ROOT_)?PASSWORD=<"                                         # #615
pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "MARIADB_ROOT_PASSWORD=.\{0,40\}"
pdftotext docs/manual/fr/marketing-brochure.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Backlog.\{0,160\}"
```

### Ce qui doit être préservé

- `#249` : `sentAt` reste un `NaiveDateTime` complet (`AAAA-MM-JJTHH:MM:SS`), jamais une date nue.
- `journal_entries::delete_all_by_company` (`journal_entries.rs:2035`), appelée par dix-huit montages de tests
  de son propre module (`grep -c "^\s*delete_all_by_company(" journal_entries.rs` → 18) et par `journal_entry_reversal_e2e.rs:905`, **ne se touche pas**.
- Les relevés datés de `docs/testing.md` gardent leurs chiffres et leurs étiquettes (consigne 15).
- Les URL `raw.githubusercontent.com/guycorbaz/kesh/v0.13.2/…` du manuel (`:225`, `:236`) : la garde G19 les tient
  égales à `\keshVersion` ; la release les portera au tag — **ne pas** les changer ici.
- La fin du paragraphe `admin-manual.tex:246` (chevrons acceptés par MariaDB) et le paragraphe `:248` — ce dernier
  **complété** par l'AC6 (chaînes 0.13.0-0.13.2) et la procédure du
  § `sec:mariadb-mdp` : vrais, inchangés.

### Exclusions écrites (périmètre minimal)

- **#612, « point voisin »** (la page des rapports charge les comptes archivés sans test Vitest) : l'issue le
  signale « non vérifié par un test … à couvrir à l'occasion » ; ce n'est pas son « À faire ». Hors périmètre ;
  pour qu'il ne se perde pas à la fermeture de #612, **l'orchestrateur ouvre une issue dédiée** (C-28-8-6).
- **#595, « envisager un vrai tableau »** : optionnel dans l'issue (« envisager ») ; la prose reste. **`409
  BANK_ACCOUNT_CLOSED`** sur ces mêmes routes (posé par la 28-4a) : écrit par la 28-4b (fiche 28-4, AC16), non ici.
- **#615 / #578** : la piste 2 de #578 (gabarit commenté) est exactement l'AC6 ; la **détection** que #578 demande
  aussi reste à #578 — la PR peut porter `refs #578`, jamais `closes`.
- **#421** : pas d'horloge simulée côté Playwright ni côté backend ; la correction tient en une ligne.
- **#561** : pas de nouveau helper de nettoyage des comptes : les montages en ont déjà (`drop_role_company`,
  `drop_company_deep`, `cleanup_test_accounts`).
- **Pas de test neuf pour #595** : la route décrite ne change pas, et le refus est défensif (aucun geste ne le
  produit) ; la chaîne est relue au code et citée (C-28-8-5).
- **Pas de capture d'écran** (B2 de l'Epic 27) : aucun contrôle visible ne change.
- **Manuels DE/IT/EN** : vides (`docs/manual/{de,en,it}/README.md`) ; rien à traduire.

### Modules touchés

1. `frontend/tests/e2e/invoices.spec.ts` — une ligne de test ;
2. `docs/testing.md` — la § *Les échecs attendus* ;
3. `crates/kesh-db/src/repositories/accounts.rs` — une suppression ;
4. `docs/api-external.md` — deux insertions ;
5. `docs/manual/fr/` — `marketing-brochure.tex`, `admin-manual.tex` et leurs PDF ;
6. `CHANGELOG.md` — une puce.

Six emplacements, trois modules au sens de la règle : cf. § *Dérogation*.

### Pièges connus du dépôt, applicables ici

- **Base remise à zéro avant chaque gate complet** (KF-039), sur la base **du worktree** seulement.
- **Gates lourds sous le verrou** (`flock …/.gate.lock`), `make` des manuels compris (consigne 13).
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **`lstlisting` et les accents** : le listing du manuel en porte déjà (« lus à ») ; rester dans les mêmes
  caractères. Un `<` ou `>` dans un `lstlisting` ne s'échappe pas ; dans la prose, `\texttt{<…>}` comme aux lignes
  voisines.
- **PDF binaires au rebase** : jamais `--ours`/`--theirs` ; régénérer après le dernier rebase (AC9).
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **Édition par l'outil Edit uniquement** (consigne 14).
- **Données réelles** : aucune donnée de `~/kesh-test/` ni du relevé de Guy dans un fichier, un test ou un commit.

### References

- Issues : [#421], [#561], [#595], [#612], [#615] ; voisines [#424] (KF-052), [#578], [#232].
- Tri : `/home/gcorbaz/devel/kesh-gate-logs/triage-issues-28.md` (hors dépôt).
- Fichier d'epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-8*.
- Fiche 28-4 : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md`, AC16 (même paragraphe
  d'`api-external.md`).
- `CLAUDE.md` : § *Test Locally First*, § *Les échecs attendus* (renvoi), § *Recompter n'est pas réécrire
  l'historique*, § *Règle de splitting préventif*, § *Issue Tracking Rule* (mots-clés sur la PR).

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 (`bmad-dev-story`, autonomie, 2026-10-14). Aucun choix nouveau : le registre `epic-28-choix-autonomes.md` (C-28-8-1 à 8) est inchangé.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/28-8/` : `build.log`, `test-fast.log`, `fe-*.log`, `m1.log`, `solo.log`, `e2e-full.log`, `make.log`.

### Completion Notes List

- **Commits** : `ebb9bc81` (#421 test), `e5370eb6` (#561), `df40a6a5` (testing.md + api-external, #421/#595), `cf4c0604` (brochure + manuel admin + PDF, #612/#615), `7d69e5ab` (CHANGELOG), puis relevé E2E de `docs/testing.md`, fiche et prompt de revue. Tous en `refs`, aucun `closes`. Base : `origin/main` = `6de5a7ff`, inchangé (aucun rebase nécessaire).
- **#421 (AC1)** : `sentAt` = `new Date(Date.now() - 3_600_000).toISOString().slice(0, 19)`. Jumeaux relevés : `git grep -n "T12:00:00" frontend/tests/e2e` ne rend plus rien ; `git grep -nE "T[0-9]{2}:[0-9]{2}:[0-9]{2}" frontend/tests/e2e` ne rend, hors fixtures CAMT, que les commentaires du site corrigé ; seul ce helper poste `reminders/manual`. Jumeau produit (`reminder-sent-at.ts`) : déjà corrigé par #259 (`CHANGELOG.md:602`), rien à changer.
- **M1** (`- 3_600_000` -> `+ 3_600_000`, appliquée : `grep -c` = 1) : `invoices.spec.ts:418` et `:442` rougissent toutes deux, `Error: manual reminder failed: 422` (`m1.log`, exit 1) ; restaurée par `git checkout -- f && touch f` ; rejeu seul des deux tests : 2 passés (`solo.log`). **Heure du run : 22:33 UTC, donc après 12:00 UTC : la preuve « matin » directe reste à faire** ; la mutation M1 est la preuve de substitution (elle rend le défaut d'origine à toute heure).
- **#561 (AC3)** : avant suppression, `git grep -nw delete_all_by_company` ne rend, hors `journal_entries.rs`, que `journal_entry_reversal_e2e.rs:905` (`journal_entries::`), des commentaires (`letterings.rs:30`, `letterings_lexical.rs:23`, `invoices.rs:4493`, migration `20260828000002`, `docs/testing.md:36`, `docs/migrations-idempotence-audit.md:86`, `CLAUDE.md:511`) et des archives `_bmad-output`. Après suppression : `cargo build --workspace --all-targets` vert, `cargo clippy --workspace --all-targets -- -D warnings` vert, `cargo fmt --check` vert.
- **#595 (AC4)** : chaîne relue au code (`supplier_invoices.rs:1343-1355`, `journal_entries.rs:2534-2549` et `2664-2677`, `errors.rs:1303-1305`), conforme à la fiche ; deux insertions ; `grep -c MATCHED_BANK_TRANSACTION docs/api-external.md` : 6 -> 8.
- **#612 (AC5)** : puce sortie du Backlog, bloc « Report à-nouveau automatique (Epic 14, v0.8, livré) » entre TVA et lettrage ; PDF aplati : bloc présent, « Backlog (Epic 13 à 15) • Budgets annuels + suivi. • Justificatifs, journaux personnalisables. ».
- **#615 (AC6)** : listing suivant `.env.example` ; prose « décommentez … » et « Décommentée sans valeur … » ; chaînes 0.13.0-0.13.2 ajoutées au paragraphe des bases déjà créées. `git grep -nE "MARIADB_(ROOT_)?PASSWORD=<" -- ':(exclude)_bmad-output'` : vide. PDF aplati : `#MARIADB_ROOT_PASSWORD=` présent, `MARIADB_ROOT_PASSWORD=<sortie` absent, `autre sortie de openssl rand` seulement dans le paragraphe des bases déjà créées. `make admin brochure` sous flock : 3 et 2 passes, stables.
- **AC2** : `docs/testing.md` mis à jour selon la fiche (chapeau, deux lignes retirées, KF-052, note datée, note historique, formule 7 à 10, annotation du tableau du 2026-08-30 laissé intact, relevé du 2026-10-13). `git grep -nE "KF-045|KF-046" -- ':(exclude)_bmad-output'` : tableau historique, son annotation, la note historique et `CLAUDE.md:706`.
- **AC7** : une puce dans la section unique `## [0.13.3] — Non publié`, liens en ligne.
- **Gates (au HEAD de code `7d69e5ab`, tous sous flock, bases `kesh_288`/`kesh_e2e_288` remises à zéro juste avant, conteneur non redémarré)** : `scripts/test-fast.sh` : fmt + clippy verts, **3603 tests exécutés, 3603 passés, 6 ignorés** (145,6 s) ; `npm run check` 0, `lint-i18n-ownership` 0, `test:unit` **146 fichiers / 1566 tests passés**, `build` 0 ; **E2E complet** port 3042 : **277 passés / 7 échoués / 19 ignorés** (11,0 min), les sept échecs sont exactement les sept KF-029 de `docs/testing.md` mis à jour (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; ni KF-052 ni pollution ; `invoices.spec.ts:418`/`:442` (ex-:415/:439) passent. Les commits suivants (relevé de `testing.md`, fiche, prompt) ne touchent aucun code ni `.tex`.
- **Aucun** `closes` dans les messages de commit ; aucune donnée réelle dans le dépôt.

### File List

`frontend/tests/e2e/invoices.spec.ts` ; `crates/kesh-db/src/repositories/accounts.rs` ; `docs/testing.md` ; `docs/api-external.md` ; `docs/manual/fr/marketing-brochure.tex` et `.pdf` ; `docs/manual/fr/admin-manual.tex` et `.pdf` ; `CHANGELOG.md` ; `_bmad-output/implementation-artifacts/28-8-petits-correctifs.md`, `28-8-review-prompt-p1.md`, `sprint-status.yaml`.

## Change Log

- **2026-10-13 — fiche créée** (`bmad-create-story`, Opus 5.5, autonomie), sur `6de5a7ff`. 9 AC, T0-T8, une
  mutation (M1), choix C-28-8-1 à 8. Validation P1 à lancer (`28-8-validate-prompt-p1.md`, Sonnet, une lentille R+F).
- **2026-10-14 — Validation P1** (`28-8-validate-prompt-p1.md`, Sonnet, **une** lentille R+F en contexte frais ;
  rapport hors dépôt `kesh-gate-logs/28-8-validate-p1.md`). **0 CRITICAL / 0 HIGH / 2 MEDIUM / 8 LOW** (RF1-RF11 ;
  RF7 est une confirmation assortie d'un numéro de ligne). La lentille confirme : les cinq issues couvertes sans
  ajout superflu ; zéro appelant d'`accounts::delete_all_by_company`, aucun import rendu inutile ; la chaîne
  `MATCHED_BANK_TRANSACTION` sur les deux routes, ses rangs, et le décompte 6 → 8 ; M1 rouge à toute heure ; les
  gardes `textes_coherents.rs` et `configuration_transmise.rs` ne rougissent pas. Remédiation (Opus 5.5), vérifiée
  au sol avant application (`git show v0.13.{0,1,2}:docs/manual/fr/admin-manual.tex`, `grep -c '^\[#[0-9]*\]:'
  CHANGELOG.md` → 0) :
  - **RF1 (MEDIUM)** : AC6 complété — le paragraphe `admin-manual.tex:248` gagne les chaînes gabarit des manuels
    0.13.0-0.13.2 ; « Ce qui doit être préservé » ajusté ; annotation de C-28-8-7.
  - **RF2 (MEDIUM)** : AC9 et pièges — PDF régénérés après le dernier rebase, jamais `--ours`/`--theirs` ; union
    étendue à `docs/testing.md`.
  - **LOW** : RF3 (bloc entre TVA et lettrage), RF4 (réserve « bouclement — écritures de clôture, report à nouveau,
    transitoires »), RF5 (preuve grep de l'AC2), RF6 (le tableau est la liste vivante), RF7 (`:1134-1159`), RF8
    (`invoice_settlements::create_in_tx`, `backup.rs`, `ensure_open_in_tx` avant les motifs), RF9 (jumeau produit
    `reminder-sent-at.ts`, déjà corrigé par #259), RF10 (liens en ligne au CHANGELOG), RF11 (dérogation : trois
    modules au sens de la règle, précaution déclarée au Project Lead) ; annotations de C-28-8-1, C-28-8-6.
  - Propagation : valeurs corrigées grepées dans la fiche et le prompt (`1158`, « avant le bloc », liens nus).
  - **Clôture** : plus aucun finding au-dessus de LOW ; la remédiation ne touche que la fiche et le registre, aucune
    ligne de code — la boucle se clôt sans passe ciblée. Statut **ready-for-dev**.

- **2026-10-14 — Développement** (`bmad-dev-story`, Sonnet 5.5) : T0-T8 faits, cf. Dev Agent Record. Statut **review** ; revue de code P1 : `28-8-review-prompt-p1.md` (Sonnet, une lentille B+E+A).

[#232]: https://github.com/guycorbaz/kesh/issues/232
[#421]: https://github.com/guycorbaz/kesh/issues/421
[#424]: https://github.com/guycorbaz/kesh/issues/424
[#561]: https://github.com/guycorbaz/kesh/issues/561
[#578]: https://github.com/guycorbaz/kesh/issues/578
[#595]: https://github.com/guycorbaz/kesh/issues/595
[#612]: https://github.com/guycorbaz/kesh/issues/612
[#615]: https://github.com/guycorbaz/kesh/issues/615
