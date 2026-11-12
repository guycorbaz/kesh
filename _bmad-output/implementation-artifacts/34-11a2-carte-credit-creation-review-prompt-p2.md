# Prompt — revue de code P2 de la Story 34-11a2 (carte de crédit, création, #972), trois lentilles, passe complète

Modèle : **Sonnet** (passe complète ; la P1 était Opus). Contexte frais. Langue du rapport : français.

Tu relis la **Story 34-11a2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**, Epic 34 :
**la carte de crédit tenue comme un compte** — création et modification d'une carte, lien **obligatoire à un passif** (relu sous verrou
partagé dans la transaction de la création, du `PUT` et du `PATCH`), **garde du numéro de carte complet**
(`kesh_core::card_number::looks_like_card_number`, IBAN valides écartés d'abord), **retypage refusé** d'un compte du plan lié à un compte
bancaire non archivé, **règlement client refusé** sur une carte, sept codes d'erreur, traces sans les chiffres, écrans, manuels. **Aucune
migration.** C'est une passe **complète** : relis toute la story, pas seulement la remédiation.

Worktree `/home/gcorbaz/devel/kesh-e34a2`, branche `story/34-11a2-carte-credit-creation`. Diffs, **aplatis** :
- la story entière, fusion de la 34-11a1 exclue : `git -C /home/gcorbaz/devel/kesh-e34a2 diff 580675bb b27c3ea1` — `580675bb` = tête de la
  34-11a1 `done`, fusionnée en `86da9022` ; `b27c3ea1` = **dernier commit de code** (remédiation de la revue P1) ;
- la remédiation seule : `git diff 3c1510d8 b27c3ea1` (`3c1510d8` = tête relue en P1).
Les commits qui suivent `b27c3ea1` ne portent que le registre, le PDF, la fiche et ce prompt.

**La revue P1** (Opus) : `/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p1-{B,E,A}.md` — 3 MEDIUM distincts (B1 = E1, B2 = E2, A1), ~27 LOW.
Sa remédiation est décrite au **Change Log** de la fiche (entrée « remédiation de la revue P1 ») et au registre (C-e34a2-10 à 15). Vérifie
chaque finding P1 au code : **corrigé, ou écarté par écrit** ; un finding déclaré corrigé et qui ne l'est pas est un finding de cette passe.

⚠️ **La base ne porte ni la 34-1 ni la 34-2** (C-e34a2-1) : quatre **raccords d'intégration**, portés par l'orchestrateur, sont écrits dans la
fiche (section *Raccords d'intégration*). Juge s'ils sont complets et si la branche ne promet plus rien qui en dépende (manuel, note ¹¹, PDF).

⚠️ **Les propriétés centrales.**
1. **Le numéro complet d'une carte n'est jamais enregistré, ni journalisé, ni tronqué en silence** : base, `audit_log`, messages d'erreur,
   champ des quatre chiffres (plus de `maxlength`), tests (numéros de test publics seulement).
2. **Une carte est toujours liée à un passif** : création, `PUT`, `PATCH` (même code, `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`, pour
   actif, charge, produit, délier), retypage ultérieur, et course avec un retypage (trois tests croisés par les routes).
3. **Un compte bancaire garde ses règles d'avant**, à deux écarts près, déclarés : l'ordre `404` avant forme du `PUT`, et deux témoins
   transitoires de la 34-11a1 récrits et renommés dans `bank_accounts_e2e.rs`.
4. **La garde ne refuse jamais un IBAN valide, une référence QR, un téléphone ni une date** ; ses faux négatifs sont **écrits** (module,
   manuel, note ¹¹).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/34-11a2-carte-credit-creation.md`
en entier (AC-1 à AC-9, tâches, mutations, *Points hérités*, *Raccords d'intégration*, **Change Log**, **Dev Agent Record**) ; les registres
`e34-choix-autonomes-e341.md` (C-e341-15, 16, 19, 21, 30 à 33, 39 à 43) et `e34-choix-autonomes-e34a2.md` (C-e34a2-1 à 15). Recoupe le Dev
Agent Record avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a2/` : `rouges.log`, `gates-3.txt`,
`gate-test-fast-3.log`, `build-3.log`, `gate-ci-3.log` (tué par `mem-guard`, 137) et `gate-ci-4.log` (rejoué), `front-*-3.log`, `e2e-r3/`
(suite complète) et `e2e-r4/` (rejeu isolé des deux échecs variables), `mut/campagne-2.log`, `mut/M*.log` (passe 2) et `mut-p1/` (passe 1) — le
script `mut-e34a2.py` et `mut-run-e34a2.sh` sont à **lire**, jamais à exécuter —, `manuels-4.log`, `flat-user-manual.txt`. **Captures** (thème
clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a2/captures/` : `34-11a2-01-creation-carte.png` à `34-11a2-06-reglement-sans-carte.png`.

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

1. **La garde du numéro complet** (`crates/kesh-core/src/card_number.rs`) : suite candidate, « entière », groupements, longueur, préfixes et
   longueurs par réseau, Luhn ; **le masquage des IBAN** (`mask_ibans`) — bornes (deux lettres bordées à gauche, séparateurs uniques,
   34 caractères), plus long préfixe valide, un mot ordinaire de deux lettres ou plus suivi de chiffres (`DE`, `IT`… : un faux IBAN valide
   est-il plausible ?), un IBAN suivi d'un numéro de carte, un numéro de carte précédé de deux lettres qui forment par hasard un IBAN valide
   (faux négatif à écrire ?) ; la liste des faux négatifs écrite (module, `user-manual.tex`, note ¹¹) contre le code.
2. **Le lien à un passif, sous verrou** : le `PATCH` lit le type du compte avant le pré-vol (C-e34a2-10) — l'ordre des erreurs d'un **compte
   bancaire** est-il exactement celui d'avant (id inconnu, archivé, d'une autre société) ? Ordre des verrous `companies → réglages → accounts →
   bank_accounts` ; les trois tests croisés (`creation_croisee_avec_un_retypage`, `put_croise_…`, `patch_croise_…`) et la sonde
   `PROCESSLIST` : déterministes ? prouvent-ils l'attente ? Le test croisé du dépôt (sonde au lieu d'une attente fixe, E8) ; la place du
   refus du retypage avant `AccountHasEntries` (testé ?).
3. **Les corps et les refus** : `cardLast4` (numéro complet → `CARD_NUMBER_NOT_ALLOWED`, `details.field = "cardLast4"`, message sans le texte),
   `provided`, ordre du `PUT` (`version < 1` → `404` → type → forme) contre le doc-comment, `api-external.md` et le CHANGELOG.
4. **Le règlement client** : refus, angle mort #1033 au manuel.
5. **Les manuels et la documentation contre le code** : `docs/manual/fr/user-manual.tex` (§ *Une carte de crédit*, l'avertissement et ses
   limites, *Plan comptable*), le **PDF** aplati (`docs/manual/fr/user-manual.pdf`), `admin-manual.tex`, `docs/api-external.md` (note ¹¹,
   tableau), `CHANGELOG.md`. Plus aucune phrase ne promet ce que la branche n'a pas (solde *Dette*, description à l'assistant) ?
6. **L'écran et les types** : `+page.svelte` (plus de `maxlength`, retrait d'un actif au passage à *Carte*), `BankAccountJournalLinkForm`,
   `SettleInvoiceDialog`, l'union discriminée de `bank-accounts.api.ts` (B10), les Vitest ajoutés (A8, A9, B9, E5).
7. **Les témoins et les mutations** : 28 mutations de la passe 2 (`mut/campagne-2.log` et les cinq frontend) ; chacune tuée par le test
   attendu (lis le `FAIL` de chaque journal) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne le diff `580675bb..b27c3ea1`, `card_number.rs` et `mask_ibans` en tête, puis les routes, le dépôt, les écrans et les tests
neufs. Cherche : un ordre de refus qui change un code existant ; un verrou pris dans un ordre inverse ; un test qui passe **à vide** ou dont la
sonde peut réussir sans l'attente qu'elle prétend constater ; un type TypeScript trop large ou trop étroit.

## Lentille E — Edge (chaque branche, chaque borne)

Carte archivée ou close (`PUT`, `PATCH`, retypage) ; `PATCH` d'une carte vers son passif courant (no-op) ; vers un passif non imputable ou
désigné créanciers ; id de carte d'une autre société ; `cardLast4` `"4111111111111111"`, `"1234 5678"`, pleine chasse ; IBAN `LI` ; IBAN
écrit en minuscules ; texte `"CH 93 0076 2011 6238 5295 7"` (séparateurs après chaque paire) ; un nom de 60 caractères qui finit sur deux lettres
et une description qui commence par des chiffres. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC-1 à AC-9 et chaque finding de la P1 : livré, testé, au lieu prescrit, ou écarté par écrit ? Les gates déclarés au Dev Agent
Record ont-ils tourné au dernier commit de code `b27c3ea1` (journaux) ? Le rejeu de `gate-ci` et le rejeu isolé des deux échecs E2E variables
sont-ils déclarés tels quels ? Les raccords d'intégration sont-ils au complet, chacun avec son test attendu ? Les issues citées (#1032, #1033) ?
Le Dev Agent Record n'affirme-t-il que ce qui a tourné, et ses décomptes, recomptés, donnent-ils le même nombre ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p2-B.md`, `/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p2-E.md`,
`/home/gcorbaz/devel/kesh-gate-logs/e34a2-review-p2-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH /
MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
