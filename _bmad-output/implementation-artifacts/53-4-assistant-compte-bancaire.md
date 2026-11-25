# Story 53-4 : l'assistant ne réécrit pas le compte bancaire (#1004)

Status: drafted

*Fiche rédigée le 2026-11-25 (Sonnet 5.5), worktree `/home/gcorbaz/devel/kesh-53-4`, branche `story/53-4-assistant-compte-bancaire`, numéros
de ligne de `4d540a6ea` (base `origin/epic/53-ecritures-justes`). Epic 53 « Écritures justes », release visée **v0.21.0** (vendredi
2026-12-04). **Aucune migration, aucun changement de contrat de l'API.** Commits en `refs #1004` ; à la PR : le mot-clé de fermeture
n'apparaît que dans le titre ou le corps de la PR (CLAUDE.md § Issue Tracking Rule).*

## Story

En tant qu'Administrateur dont l'installation est restée à l'étape 6 de l'assistant alors qu'un compte bancaire principal existe déjà,
je veux que l'étape reparte de ce compte (banque, IBAN, QR-IBAN, description, compte lié), pour que la renvoyer ne vide pas sa
description, n'efface pas son QR-IBAN et ne remplace pas son compte du plan par une présélection, sans que je l'aie demandé.

## Étape 0 — prémisses relues au code (à rejouer au développement)

Le constat de #1004 (« l'écran ne préremplit rien ; un nouvel envoi efface la description et remplace le lien par la présélection »)
est **confirmé pour l'écran** ; il est **démenti pour le serveur**, qui fait déjà ce qu'on lui demande. Un troisième champ s'ajoute à
ceux que l'issue nomme.

1. **Le serveur ne réécrit que ce qu'on lui envoie.** `set_bank_account` (`crates/kesh-api/src/routes/onboarding.rs:635-746`) passe
   à `upsert_primary_in_tx` (`crates/kesh-db/src/repositories/bank_accounts.rs:389-470`) un `PrimaryDetails` à **doubles options**
   (`:317-322`) : `description` et `journalAccountId` **absents = inchangés** (l'`UPDATE` ne nomme pas la colonne, `:422-430`),
   **`null` = effacés**, valeur = posée. Le court-circuit no-op (`is_no_op_change`, `:285-298`) ne compare que les champs présents.
   C'est le contrat de la Story 34-1 (C-e341-34), éprouvé par `bank_step_replays` et `bank_step_update_without_new_fields_keeps_them`
   (`crates/kesh-api/tests/onboarding_bank_account_e2e.rs:321`, `:492`). **Le serveur n'a aucun défaut ici.**
2. **L'écran envoie toujours des valeurs, jamais « absent ».** `submitBankAccount`
   (`frontend/src/routes/onboarding/+page.svelte:474-505`) envoie `description: bankDescription.trim() || null` et
   `journalAccountId: bankJournalAccountId`, et `setBankAccount` (`onboarding.api.ts:80-87`) recopie les cinq champs. Les états
   `bankName`, `bankIban`, `bankQrIban`, `bankDescription` partent de `''` (`:230-248`) et `bankJournalAccountId` de `null` ; la
   **seule** lecture du serveur à l'étape 6 est la liste du plan (`loadBankLedgerOptions`, `:270-297`), qui présélectionne le `1020`
   (`preselectedBankLedgerId`) sans jamais lire un compte bancaire. Sur un compte principal existant, l'envoi écrit donc : description
   `null`, lien = présélection, QR-IBAN `null` si le champ est resté vide.
3. **Inventaire des champs de `bank_accounts` écrits par cet `UPDATE`** (`:422-430`, colonnes de `columns!()`, `:52-56`) :

   | Colonne | Écrasée par l'envoi de l'écran ? | Source de l'écrasement |
   |---|---|---|
   | `description` | **oui**, vidée (`null`) | l'écran n'a pas de valeur de départ |
   | `journal_account_id` | **oui**, remplacée par la présélection (1020 s'il existe) ou `null` | idem |
   | `qr_iban` | **oui**, effacée si le champ est resté vide — **non nommé par #1004** | `qr_iban` n'est pas une double option : absent et `null` valent « effacé » (`routes/onboarding.rs:600-604`, `:672-679`) |
   | `bank_name`, `iban` | réécrites par la valeur saisie (obligatoires à l'écran, donc retapées) ; sans effet si elles sont identiques | le formulaire démarre vide |
   | `version` | +1 si un champ diffère (verrou optimiste) | — |
   | `account_kind`, `card_last4`, `is_primary`, `archived`, `closed_on`, `provider`, `provider_account`, `fee_account_id`, `company_id` | non : l'`UPDATE` ne les nomme pas | — |

   Les écritures comptables ne sont pas touchées (l'issue le dit ; vérifié : `upsert_primary_in_tx` n'écrit que `bank_accounts`, plus
   l'audit `bank_account.updated` et `installation.step_completed`). Ce que l'écrasement change dans les livres, c'est **le compte du
   plan par lequel passeront les écritures de réconciliation et d'import** : d'où le P1 de l'epic.
4. **Quand le cas se produit.** `lock_state_at_step(&mut tx, 6, true)` ne laisse passer l'étape qu'à `step_completed = 6`, et l'écran
   n'a **aucun bouton de retour** (`grep -n -i "précédent\|retour\|back" +page.svelte` ne rend rien d'utile). Le compte principal
   ne précède donc l'étape 6 que s'il a été créé **ailleurs** pendant l'assistant : `POST /api/v1/bank-accounts` depuis une clé API ou un
   autre onglet, restauration ou import d'installation tombant à l'étape 6. **À sonder au développement** (prémisse non établie
   par exécution : cette fiche ne lance ni serveur ni `curl`, § 2.1 de `docs/agents-orchestration.md`) : amener une installation de test à
   l'étape 6, créer un compte principal par l'API, recharger `/onboarding`, et constater les trois effacements ci-dessus. Les tests
   de l'étape (`set_step(&pool, 6)` après un premier envoi) simulent ce retour ; ils prouvent le serveur, non l'accessibilité du cas.
5. **La lecture du compte existant est possible à l'étape 6** : `GET /api/v1/companies/current` (`lib.rs:1322`, handler
   `routes/companies.rs:650-665`) rend `bankAccounts` avec `bankName`, `description`, `iban` (entier), `qrIban`, `accountKind`,
   `provider`, `isPrimary`, `journalAccountId` (`BankAccountJson`, `:607-629`) ; `fetchCompanyCurrent` (`settings.api.ts:26`) l'appelle
   déjà au récapitulatif. ⚠️ `list_by_company` (`Default::default()`) **exclut les comptes archivés et clos** alors que le `SELECT … FOR
   UPDATE` de l'upsert ne filtre que `archived = FALSE` : un principal clos (`closed_on`) serait réécrit sans que l'écran l'ait vu.
   **À trancher au développement** (D3) et à prouver par un test.

## Décisions

- **D1 — le correctif est à l'écran, le serveur ne change pas.** Le contrat « absent = inchangé, `null` = effacé » est publié (manuel,
  note d'API, CHANGELOG de la 0.17.6) et testé ; le rendre « `null` = inchangé » casserait le client qui veut effacer. *Écartée* : un
  garde serveur qui refuserait d'effacer une description existante (casse l'effacement voulu, que le test (e) de `bank_step_replays` fixe).
- **D2 — l'écran repart du compte existant.** À l'étape 6, il lit `fetchCompanyCurrent()` une fois, retient **le compte principal
  ordinaire** (`isPrimary && provider === null && accountKind === 'bank'`) et en préremplit les cinq champs. Un avis (`data-testid=
  "onboarding-bank-existing-notice"`) dit « Un compte bancaire principal existe déjà : le formulaire en reprend les valeurs. » La
  présélection du `1020` ne vaut que **sans** compte principal (compte neuf) : `bankLedgerTouched` n'y change rien, la valeur chargée
  prime sur la présélection. *Écartée* : omettre `description` et `journalAccountId` tant que l'utilisateur n'y a pas touché (« absent =
  inchangé ») : correct pour ces deux champs, mais laisse l'écran vide, le QR-IBAN effacé (non double option), et l'utilisateur sans
  moyen de voir ce qui est conservé.
- **D3 — ce que le serveur ne peut pas ignorer, l'écran le garde.** Trois garde-fous d'écran :
  1. *Le lien du compte existant ne figure pas dans la liste des comptes liables* (compte archivé, devenu non imputable, ou de rôle
     `Receivable` / `Payable`, que le serveur refuserait en `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`) **ou la liste n'a pas pu être
     chargée** : le sélecteur affiche « Non configuré » **et l'écran n'envoie pas `journalAccountId`** (absent = inchangé), avec une phrase
     sous le champ (« Le compte lié actuel est conservé. »). Il ne l'envoie pas à `null`.
  2. *La lecture du compte existant échoue* : l'étape ne s'envoie pas (bouton désactivé, motif sous le formulaire, bouton *Réessayer*).
     Un échec réseau ferait échouer l'envoi de toute façon ; et un envoi à l'aveugle effacerait le QR-IBAN (§ Étape 0, point 3).
     *Écartée* : laisser l'étape utilisable comme pour la liste du plan (AC-6 de la 34-1) : la liste du plan ne perd rien à échouer, la
     lecture du compte, si.
  3. *Le principal est clos* (`closedOn`) ou absent de la liste alors que l'upsert le trouverait : **le développeur tranche en lisant
     `list_by_company` et `upsert_primary_in_tx`**, l'écrit au Dev Agent Record, et ouvre une issue (« une issue par problème ») si la
     divergence est réelle. Ce n'est pas corrigé dans la story : l'écran affiche un compte neuf, comme aujourd'hui.
- **D4 — le serveur gagne un test, pas une ligne.** Un test pose, sur un compte existant, le corps exact que l'écran enverra après
  D2 (les cinq champs lus du compte) et prouve `Unchanged` : version et trace inchangées. Et un test épingle que `qrIban` **absent** ou
  `null` efface le QR-IBAN d'un compte existant (contrat actuel, que l'écran doit respecter) : si un jour le contrat change, le test le dit.
- **D5 — une seule route de lecture.** Pas de nouveau point d'entrée serveur ; `fetchCompanyCurrent` est réutilisé (DRY).

## Acceptance Criteria

1. **AC-1 — l'écran repart du compte existant.** À l'étape 6 avec un compte principal ordinaire existant, `#bank-name`, `#bank-iban`,
   `#bank-qr-iban`, `#bank-description` et `#bank-journal-account` portent les valeurs de ce compte ; l'avis
   `onboarding-bank-existing-notice` s'affiche. Sans compte principal : écran vide, présélection du `1020` s'il existe, pas d'avis
   (comportement de la 34-1, inchangé).
2. **AC-2 — rien n'est écrasé sans le dire.** Renvoyer l'étape **sans rien changer** sur un compte existant (description, lien et QR-IBAN
   posés) rend 200, `step_completed = 7`, et le compte est identique : mêmes valeurs, **même `version`**, aucune entrée
   `bank_account.updated` (seulement `installation.step_completed`). Modifier **un** champ ne change que celui-là.
3. **AC-3 — la valeur chargée prime sur la présélection.** Un compte existant lié à un compte autre que le `1020` reste lié à celui-ci à
   l'écran et à l'envoi ; un choix fait dans le sélecteur avant la fin du chargement ne se perd pas (garde de la 34-1, conservée).
4. **AC-4 — le lien non proposable est conservé, non effacé.** Lien du compte existant absent de la liste (archivé, non imputable, rôle
   `Receivable` / `Payable`) ou liste en échec : le corps de l'envoi **ne porte pas** `journalAccountId`, le lien en base est inchangé, et
   l'écran dit « conservé ».
5. **AC-5 — lecture du compte en échec.** `fetchCompanyCurrent` rejette : l'étape ne s'envoie pas, le motif s'affiche sans toast (garde
   `onboarding-catch-guard.test.ts`), *Réessayer* relance la lecture et, si elle réussit, l'étape redevient utilisable.
6. **AC-6 — le compte neuf ne change pas.** Sans compte principal, le corps envoyé est celui de la 34-1 (description normalisée ou `null`,
   lien présélectionné ou `null`) ; les AC de la 34-1 (liste filtrée, présélection, « modifiable plus tard ») restent vrais.
7. **AC-7 — un compte de prestataire ou une carte n'est pas repris.** Un compte principal `provider = 'paypal'` ou `accountKind =
   'credit_card'` n'est pas pris pour le compte de l'assistant : ce cas est déclaré inatteignable par le schéma (`chk_bank_accounts_card_shape`)
   ou tranché par D3 (3) au développement, dit au Dev Agent Record.
8. **AC-8 — quatre locales, manuel.** Les trois clés neuves existent en FR, DE, IT, EN (§ Sites) ; le manuel d'utilisation dit le
   comportement (FR ; DE/EN/IT s'ils portent la phrase de l'étape 6) ; les PDF se régénèrent à la release, non dans la story.
9. **AC-9 — recette visuelle** de chaque état touché, thème clair, sur base de démonstration : captures jointes au Dev Agent Record et
   au prompt de revue (§ Recette visuelle).
10. **AC-10 — le serveur est inchangé.** `git diff origin/epic/53-ecritures-justes -- crates/*/src/*` est vide ; les tests Rust neufs
    passent sans l'avoir touché.

## Tâches

- [ ] **T0 — sonder (D3, § Étape 0 point 4)** : sur une installation jetable (base `kesh_534`, jamais celle du NAS), amener l'étape 6,
  créer un principal par `POST /api/v1/bank-accounts`, relever par l'écran actuel les trois effacements ; rouge de référence au
  journal `kesh-gate-logs/e534/sonde-t0.txt`. Lire `list_by_company` contre le `SELECT … FOR UPDATE` (D3-3).
- [ ] **T1 — Rust, tests seuls** (`crates/kesh-api/tests/onboarding_bank_account_e2e.rs`) : (a) `bank_step_replay_with_full_screen_body_is_unchanged` : créer
  un compte par l'étape (description « Personnel », lien 1020, QR-IBAN valide), `set_step(6)`, renvoyer les cinq champs identiques :
  `Unchanged`, `version` 1, actions `["installation.step_completed"]` ; (b) `bank_step_absent_or_null_qr_iban_erases_it` : même point de
  départ, corps sans `qrIban` puis avec `qrIban: null` → QR-IBAN `NULL`, `Updated`. Rouges attendus : aucun (le serveur est juste) ;
  la preuve est la mutation M1.
- [ ] **T2 — l'écran** (`+page.svelte`, éventuellement `onboarding.svelte.ts`) : état `existingPrimary`, lecture une fois à
  `stepCompleted === 6` (même patron que `bankLedgerRequested` : drapeau non réactif), garde « pas d'envoi avant la fin de la lecture » (comme
  `bankLedgerLoading`), préremplissage des cinq champs, priorité de la valeur chargée sur `preselectedBankLedgerId`, omission de
  `journalAccountId` selon AC-4. `BankAccountStepPayload.journalAccountId` devient `number | null | undefined` (absent = inchangé) ;
  `setBankAccount` n'ajoute la clé que si elle est définie. Le tri « compte principal ordinaire » vit dans une fonction pure de
  `lib/features/bank-accounts/format.ts` (testée), pas inline.
- [ ] **T3 — Vitest** (`frontend/src/routes/onboarding/onboarding-bank-step.test.ts`, `bank-accounts/format.test.ts`) : AC-1, AC-3, AC-4,
  AC-5, AC-6, AC-7 ; le test de non-écrasement (AC-2 côté écran) vérifie le **corps envoyé** à `api.setBankAccount`.
- [ ] **T4 — Playwright** (`frontend/tests/e2e/onboarding-bank-account.spec.ts`, bloc neuf, patron de `beforeEach` existant) : amener
  l'étape 6 par l'API, créer un principal par `POST /api/v1/bank-accounts` (description, lien non 1020, QR-IBAN), recharger, constater le
  préremplissage, soumettre sans rien changer, relire par `GET /api/v1/bank-accounts` : description, lien, QR-IBAN et `version` identiques.
  Sélecteurs d'`id`, aucun sélecteur de texte. Spec jouée **seule** d'abord, puis suite complète au dernier commit de code.
- [ ] **T5 — i18n et manuel** : trois clés × 4 locales ; `user-manual.tex` (§ Sites) ; Notes de version ci-dessous.
- [ ] **T6 — mutations (M1 à M6)** et grep du symptôme, journal par mutation.
- [ ] **T7 — recette visuelle** et Dev Agent Record.

## Dev Notes

- **Sites et ordre de lecture.**
  `crates/kesh-api/src/routes/onboarding.rs:600-616` (struct), `:635-746` (handler) — *lus, non modifiés* ;
  `crates/kesh-db/src/repositories/bank_accounts.rs:285-298`, `:317-322`, `:389-470` — *lus, non modifiés* ;
  `frontend/src/routes/onboarding/+page.svelte:229-303` (états et liste du plan), `:474-505` (`submitBankAccount`), `:781-875` (formulaire) ;
  `frontend/src/lib/features/onboarding/onboarding.api.ts:63-87` ; `onboarding.svelte.ts:134-142` ;
  `frontend/src/lib/features/settings/settings.api.ts:26` (`fetchCompanyCurrent`) ; `frontend/src/lib/features/bank-accounts/format.ts`
  (fonction pure neuve) ; catalogues `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` près de `onboarding-bank-editable-later`
  (`:3540` en FR).
- **Clés i18n neuves** : `onboarding-bank-existing-notice`, `onboarding-bank-existing-load-error`, `onboarding-bank-ledger-kept`.
  Un repli par clé (gardes `i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts`) ; recompter `sitesTotal` de
  `i18n-keys.test.ts` et le compteur de `i18n-libelle-en-dur.test.ts` après coup, depuis la source. Si le test d'appartenance
  (`npm run lint-i18n-ownership`) exige un bloc propre, y ranger les trois clés ; ne pas mêler au bloc d'un autre epic.
- **Le sélecteur** est un `<select bind:value>` sur `number | null` : une valeur chargée qui n'est pas une option affiche la première
  (« Non configuré »). D'où l'état séparé « lien conservé non proposable » (AC-4) : ne pas laisser le `bind` convertir en `null`.
- **IBAN à l'écran** : le compte existant le rend en entier ; le préremplir dans un champ de saisie est voulu (le récapitulatif de
  la 36-17 ne montre jamais d'IBAN, ce n'est pas la même surface). Aucun IBAN dans une capture versée à une issue publique : captures
  hors dépôt, IBAN de démonstration de la suite.
- **Aucun mot-clé de fermeture** dans les commits ; `refs #1004`. Priorité de l'issue : le corps dit P2, les labels P1, l'epic P1 ; la
  PR reprend P1 (l'effet est un compte du plan changé sans le dire).
- **Défauts voisins** : si la sonde T0 montre que le cas n'est atteignable que par l'API, l'issue en dit la portée réelle (n'est pas
  fermée pour autant : l'écran réécrivait bel et bien). Tout autre écart (principal clos, D3-3) s'ouvre en issue, sans regroupement.
- **Agents** : le développeur ne touche ni la base `kesh_dev` ni le conteneur ; bases `kesh_534` et `kesh_e2e_534`, port à demander à
  l'orchestrateur ; journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e534/`.

## Tests et mutations

Chaque mutation : appliquer, lancer le seul test nommé, constater le rouge, `git checkout --` **puis `touch`** (cargo garde sinon le
binaire muté), journal `kesh-gate-logs/e534/mut/Mn.log`.

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | `is_no_op_change` ignore la description (`is_none_or` → `true`) | `bank_step_replays` (d) : déjà tuée en 34-1 (M8) ; rejouée ici pour établir que T1 (a), qui compare aussi `version`, ne passe pas à vide |
| M2 | l'écran ne lit pas le compte existant (T2 sans l'appel) | Vitest AC-1, Playwright T4 |
| M3 | la présélection du `1020` écrase la valeur chargée | Vitest AC-3 |
| M4 | lien absent de la liste → envoyé à `null` au lieu d'être omis | Vitest AC-4 (corps sans clé `journalAccountId`) |
| M5 | l'écran n'attend pas la lecture avant d'envoyer | Vitest AC-5 (bouton désactivé) |
| M6 | `bankQrIban` non prérempli | Vitest AC-1 (valeur de `#bank-qr-iban`) ; Playwright T4 (QR-IBAN relu inchangé) |

Les tests Rust T1 n'ont pas de rouge à produire (le serveur est juste, AC-10) : leur valeur est de fixer le contrat que l'écran doit respecter. Une mutation qui survit est écrite
comme telle, avec sa raison (une équivalente se démontre, non se suppose).

## Manuel

`docs/manual/fr/user-manual.tex:225` (étape *Compte bancaire* de l'assistant) et `:2948` (§ description) : ajouter, après « Cette étape
peut aussi être passée… », une phrase : « Si un compte bancaire principal existe déjà quand vous arrivez à cette étape, le formulaire
en reprend les valeurs ; le renvoyer tel quel ne change rien. » Vérifier DE/EN/IT : `grep -n "présélectionné\|preselected\|vorausgewählt\|preselezionat"
docs/manual/*/user-manual.tex` ; ajouter la phrase aux langues qui portent l'étape 6. **PDF** : relus aplatis (`pdftotext f.pdf - | tr '\n'
' ' | tr -s ' '`) pour chercher toute phrase que la story rend fausse (« se modifient ensuite », « une simple proposition de l'écran ») ; **non
commités** (CLAUDE.md § Fusions et fichiers communs).

## Recette visuelle (CLAUDE.md § *Recette visuelle*)

Base de démonstration jetable (`kesh_e2e_534`), thème clair, captures sous `/home/gcorbaz/devel/kesh-gate-logs/e534/captures/` :

1. `01-compte-neuf.png` — pas de principal : formulaire vide, 1020 présélectionné, phrase « modifiable plus tard », **pas d'avis**.
2. `02-compte-existant.png` — principal avec description, lien ≠ 1020, QR-IBAN : cinq champs préremplis, avis visible.
3. `03-lien-conserve.png` — lien du compte existant non proposable : « Non configuré » et la phrase « conservé ».
4. `04-lecture-en-echec.png` — lecture refusée : motif, bouton *Réessayer*, étape non envoyable.
5. `05-chargement.png` — pendant la lecture : bouton désactivé.
6. `06-apres-envoi.png` — la page « Configuration terminée », puis la page *Comptes bancaires* montrant description et lien conservés.

## Notes de version

*(Pour l'assemblage du CHANGELOG ; sans numéro d'epic.)* **Corrigé** — L'assistant de démarrage ne réécrit plus un compte bancaire principal
qui existe déjà : si on arrive à l'étape du compte bancaire alors qu'un compte principal existe (créé entre-temps par un autre onglet ou
par l'API), le formulaire en reprend la banque, l'IBAN, le QR-IBAN, la description et le compte comptable lié, au lieu de les remplacer
par un formulaire vide et par la présélection du compte 1020. Sans compte existant, rien ne change. Aucune migration ; l'API
`POST /api/v1/onboarding/bank-account` est inchangée.

## Questions pour le Project Lead

1. **Atteignabilité.** L'écran n'a pas de retour en arrière : le cas ne se présente qu'avec un compte créé hors de l'assistant. La
   story reste utile (l'écran écrase) ; veux-tu quand même la priorité P1 de l'epic, ou P2 comme le corps de l'issue ?
2. **QR-IBAN.** Il est effacé comme la description, ce que l'issue ne dit pas. La story le traite par l'écran ; veux-tu en plus le
   passer en « absent = inchangé » côté serveur (contrat 0.17.5 modifié, `b` monte) ? Recommandation : non.
3. **Lecture en échec.** D3-2 bloque l'étape plutôt que de la laisser envoyer à l'aveugle ; accord ?

## Dev Agent Record

*(à remplir au développement)*

## Change Log

- 2026-11-25 : fiche rédigée (Sonnet 5.5), `drafted`. Validation P1 : prompt `53-4-assistant-compte-bancaire-validation-prompt-p1.md`, passe
  complète, **Opus**, contexte frais.
