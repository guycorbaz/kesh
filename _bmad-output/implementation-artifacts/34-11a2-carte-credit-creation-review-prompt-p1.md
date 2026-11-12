# Prompt — revue de code P1 de la Story 34-11a2 (carte de crédit, création, #972), trois lentilles, première passe complète

Tu relis la **Story 34-11a2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**, Epic 34 :
**la carte de crédit tenue comme un compte** — création et modification d'une carte (`accountKind`, `cardLast4`, IBAN facultatif dans les
corps), lien **obligatoire à un passif** (création, `PUT`, `PATCH`, relu sous verrou partagé dans la transaction), **garde du numéro de carte
complet** (`kesh_core::card_number::looks_like_card_number`, nom et description, deux types, assistant compris), **retypage refusé** d'un compte
du plan lié à un compte bancaire non archivé (`accounts::update`, `FOR UPDATE` en tête, route rejouée), **règlement client refusé** sur une
carte, sept codes d'erreur, traces sans les chiffres, écran *Comptes bancaires*, sélecteurs, manuels. **Aucune migration** (le schéma est
celui de la 34-11a1).

Worktree `/home/gcorbaz/devel/kesh-e34a2`, branche `story/34-11a2-carte-credit-creation`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e34a2 diff 27ca0bd9 <DERNIER_COMMIT_DE_CODE>` (`27ca0bd9` = base, tête de la 34-11a1). Langue du rapport :
français. Modèle : **Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

⚠️ **La base ne porte ni la 34-1 ni la 34-2** (C-e34a2-1) : `linkableLedgerAccounts` et le solde d'ouverture n'y existent pas. Ce qui en
dépend (AC-7 « sens *Dette* présélectionné », la description de l'assistant) est **renvoyé à l'intégration** et le dit ; juge si ce renvoi est
honnête et complet, non s'il est fait.

⚠️ **Les propriétés centrales.**
1. **Le numéro complet d'une carte n'est jamais enregistré, ni journalisé** : ni dans la base, ni dans `audit_log` (`card_last4_present`,
   jamais les chiffres), ni dans un message d'erreur, ni dans un test (numéros de test publics des réseaux seulement).
2. **Une carte est toujours liée à un passif**, et rien ne la laisse liée à autre chose : ni le `PATCH`, ni un retypage concurrent, ni un
   retypage ultérieur (C-e341-16, 31, 40).
3. **Un compte bancaire garde exactement ses règles d'avant** : `bank_accounts_e2e.rs` vert sans modification, payload d'écran inchangé.
4. **La garde ne refuse jamais un IBAN, une référence QR, un téléphone ni une date** (C-e341-15, arbitrage de l'orchestrateur).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/34-11a2-carte-credit-creation.md`
en entier (Choix, AC-1 à AC-9, T1 à T8, mutations, *Points hérités*, **Change Log** et **Dev Agent Record**) ; les registres
`e34-choix-autonomes-e341.md` (C-e341-15, 16, 19, 21, 30 à 33, 39 à 43) et `e34-choix-autonomes-e34a2.md` (C-e34a2-1 à 9). Le dernier rapport
de validation de la fiche : `/home/gcorbaz/devel/kesh-gate-logs/e34-validate-p6-e341.md` (F1 à F4, LOW). Ne prends pas le Dev Agent Record pour
argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a2/` : `rouges.log`,
`cible-*.log`, `gate-*.log`, `vitest-*.log`, `front-*.log`, `e2e-*/`, `mut/M*.log` (le script `mut-e34a2.py` et `mut-run-e34a2.sh` sont à
**lire**, jamais à exécuter), `manuels-*.log`. **Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a2/captures/` :
`34-11a2-01-creation-carte.png`, `34-11a2-02-refus-numero-complet.png`, `34-11a2-03-liste.png`, `34-11a2-04-modification-carte.png` —
regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La garde du numéro complet** (`crates/kesh-core/src/card_number.rs`) contre C-e341-15 (c) à (g) : la définition de la suite (séparateur
   **unique** entre deux groupes, double séparateur ou séparateur de queue qui termine), « entière » (caractère avant le premier chiffre et après
   le dernier), groupements, longueur, préfixes et longueurs par réseau (relis la table, borne par borne), Luhn. Calcule à la main, au besoin, la
   somme de Luhn des exemples ; cherche un faux positif usuel (numéro de TVA `CHE-123.456.789`, numéro AVS `756.1234.5678.97`, référence de
   facture, montant `1 234 567.89`, date `2026 11 12`, numéro de compte postal `10-12345-6`). Les tests négatifs de la famille (i) passent-ils
   **vraiment** Luhn (assertion présente) ? Appels : `validate_bank_input` (nom, puis description normalisée), `onboarding::set_bank_account`.
2. **Le lien à un passif, sous verrou** : ordre des verrous `companies → réglages → accounts → bank_accounts` à la création, au `PUT`, au
   `PATCH` (`recheck_ledger_type_in_tx`, `locked_ledger_type`, `ensure_kind_admits`) ; le retypage (`accounts::update`, `FOR UPDATE` en tête,
   `first_linked_bank_account_refusing` sous `LOCK IN SHARE MODE`, placé avant `AccountHasEntries`). Un cycle d'interblocage nouveau est-il
   possible, et la route `PUT /accounts/{id}` rejouée le couvre-t-elle ? Le test croisé `retypage_croise_avec_la_creation_d_une_carte` est-il
   déterministe, et prouve-t-il ce qu'il dit (M8) ? Le `PATCH` contrôle la carte **après** `set_journal_account_id_for_company` : le refus
   annule-t-il bien l'`UPDATE` (`conclure`), et le no-op d'un `PATCH` identique d'une carte reste-t-il un no-op ?
3. **Les corps et les refus** : `iban: Option<String>` — un compte bancaire sans IBAN rend-il `400` et non `500` ? « Fourni » (`provided`) pour
   `cardLast4`, `iban`, `qrIban` : `""` et espaces ; `cardLast4` pleine chasse (`１２３４`) ou à espaces internes ; l'ordre des erreurs du `PUT`
   (C-e341-43 : `404` avant toute forme dépendant du type — et `version < 1` ?) contre le doc-comment ; `NewBankAccount.iban` vide au `PUT`
   (M9b) ; `BankAccountPutFields::default()` ; les traces (`card_last4_present`, `account_kind` dans `before`/`after`).
4. **Le règlement client** (`invoice_settlements_write::settle_invoice`) : refus après « clos », avant toute écriture ; l'angle mort assumé
   (proposition « facture » de la réconciliation) est-il vrai au code et écrit au manuel ? Le paiement fournisseur depuis une carte (test).
5. **Les manuels et la documentation contre le code** : `docs/manual/fr/user-manual.tex` § *Une carte de crédit* (`sec:carte-credit`), la
   phrase récrite de *La description d'un compte*, le paragraphe du *Plan comptable* sur le retypage ; `docs/manual/fr/admin-manual.tex`
   § *Un compte sans IBAN* ; `docs/api-external.md` (note ¹¹ du § 7, tableau des erreurs) ; `CHANGELOG.md` § `[0.17.6] — Non publié`.
   **Le PDF n'est pas régénéré dans la story** (régénéré à l'intégration, consigne de l'epic) : contrôle le `.tex`, et dis si le PDF du dépôt
   ment désormais. Chaque phrase est-elle vraie au code (par exemple : « le lien d'une carte se change, il ne se retire pas » ; « la garde vaut à
   l'onboarding » ; « Solde d'ouverture … avec le sens *Dette* », que cette branche **n'implémente pas** — C-e34a2-1) ?
6. **L'écran** : `routes/(app)/bank-accounts/+page.svelte` (type à la création, champs masqués, payloads par type, type figé en édition),
   `BankAccountJournalLinkForm.svelte` (passifs seuls, pas de *Délier* pour une carte), `SettleInvoiceDialog.svelte` (cartes écartées avant le
   filtre du compte débiteurs), `credit-card.ts`. Un `$derived` qui lirait un état non réactif ; un payload de compte bancaire qui aurait changé.
7. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`mut/M*.log`) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `card_number.rs`, les deux corps, `validate_bank_input`, `validate_journal_account_id`, `recheck_ledger_type_in_tx` et ses
deux aides, `create_once`, `update_bank_account`, `update_once`, `patch_once`, `accounts::update` et `first_linked_bank_account_refusing`,
`type_in_share_mode`, `update_for_company`, `settle_invoice`, les mappages d'erreur, `onboarding::set_bank_account`, `routes/accounts.rs`, les
écrans. Cherche : un `Option` aplati en `""` ; un ordre de refus qui change un code d'erreur existant ; un verrou pris dans un ordre inverse ; un
test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Carte **archivée** ou **close** (`PUT`, `PATCH`, retypage) ; passif lié **archivé** ou **non imputable** ; passif devenu compte de créance
désigné ; deux cartes sur le même passif ; carte et compte bancaire sur le même compte du plan, puis retypage en actif ; `PUT` d'une carte qui
renvoie `"accountKind": "credit_card"` ; `PUT` avec `iban: ""` sur une carte ; `cardLast4` `"0000"` ; un nom de 60 caractères qui contient un
numéro ; une description de compte existant qui en contient un (refus au prochain `PUT`, écrit au manuel ?) ; l'assistant appelé après
l'onboarding (le refus vient-il avant le contrôle d'étape ?). Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par
`grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC-1 à AC-9 : livré, testé, au lieu prescrit ? Les mutations nommées (M1 à M10b) ont-elles été jouées et tuées par le test
attendu ? Les gates déclarés au Dev Agent Record ont-ils tourné, au dernier commit de code (journaux) ? Les points hérités (E5/B7, B5, E6, E3,
B3) : traités ou écartés par écrit ? Les écarts (C-e34a2-1, C-e34a2-9 à la consigne Edit) : déclarés ? Les issues demandées : une par problème ?
Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p1-B.md`, `/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p1-E.md`,
`/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH /
MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
