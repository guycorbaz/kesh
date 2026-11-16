# Prompt — revue de code P1 de la Story 35-3 (#537), trois lentilles, première passe complète

Tu relis la **Story 35-3** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **un compte désigné aux réglages de facturation (créance, TVA due, créanciers, TVA récupérable) et retypé
depuis sa désignation ne reçoit plus d'écriture** ; la validation d'une facture client ou fournisseur le refuse en
`400 DESIGNATED_ACCOUNT_WRONG_TYPE`, nommé, typé, traduit (quatre langues), sans rien écrire ni consommer de numéro. **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-e353`, branche `story/35-3-comptes-designes-type`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e353 diff 049b0466 df8ea143` (`049b0466` = base ; `df8ea143` = dernier commit de code ; `5500570b` ne porte
que le compte rendu). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-3-comptes-designes-type.md` en entier (D1 à D6, AC1 à AC14, étape 0, tests, *Ce qui doit rester
vrai*, **Change Log** et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e350.md` (C-e350-2, C-e350-8) et
`e35-choix-autonomes-e353.md` (C-e353-1 à 3). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors
dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e353/` : `etape0.log`, `recensement-set.txt`, `recensement-verdicts.md`,
`rouges.log`, `rouge-vitest.log`, `verts-cibles.log`, `gates.log`, `gate-test-fast.log`, `gate-ci.log`, `front-*.log`, `e2e-r1.log`,
`mutations.log`, `mut/`, `make-fr.log`, `*-plat.txt` (les scripts `*.sh`, `*.mjs`, `*.py` sont à **lire**, jamais à exécuter). **Captures**
(thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e353/captures/` — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
  aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La garde à l'usage** (`check_written`, `company_invoice_settings.rs`) : le type lu sous le **même verrou** et dans la même requête (D5,
   `account_type` en dernière colonne, `FORCE INDEX (PRIMARY)`, `LOCK IN SHARE MODE`) ; les sondes de `invoices_validate_vat.rs`,
   `supplier_invoices_repository.rs`, `rejeu_interblocage_e2e.rs` reconnaissent-elles toujours l'accesseur, et les deux sondes
   `account_type, active FROM accounts` ne le reconnaissent-elles pas ? L'ordre par **accumulation** (D3) : absent/archivé > type >
   non imputable, sans retour anticipé ; un compte retypé et non imputable nommé par le seul refus de type ; dédoublonnage d'un compte qui
   tient deux rôles ; tri par **numéro** de `details.rejected` et des phrases. `LockedDesignatedAccount` gagne un champ : l'avoir
   (`credit_notes.rs`) ne change pas de comportement (les avoirs ne sont pas soumis au contrôle — vrai au code ?).
2. **Une seule source des types attendus** (D4) : la constante publique, lue par `check_written` (via `DesignatedRole::expected_type`) **et**
   par la route `PUT /company/invoice-settings` ; reste-t-il un `AccountType::Asset`/`Liability` codé en dur pour ces rôles ?
   `accepts_account_type` (qui admet le passif pour la créance, #537) est-il cohérent avec la constante, ou contradictoire ?
3. **Rien n'est écrit sur refus** : facture en brouillon, aucune écriture, séquence de numérotation inchangée — côté client et côté
   fournisseur (numéro interne `FF-…` de la 33-1 compris). Le refus vient-il avant toute écriture dans les deux chemins ?
4. **Le message et l'i18n** : `t_args`, une phrase par compte puis la phrase d'action, séparateur une espace, aucune clé à sélecteur ;
   les dix clés ×4 locales, placées à côté de `error-designated-account-not-postable` ; replis Rust égaux au fr-CH, inscrits à **G9**
   avec leurs constantes de substitution, apostrophe typographique. C-e353-2 (libellés de la page plutôt que ceux de D2) : les libellés
   sont-ils exactement ceux de la page des réglages dans les quatre langues ? L'anglais et son article.
5. **Le contrat** : mappage `DbError` → `AppError` → 400, `details.rejected[{accountId, accountNumber, setting, expectedType, actualType}]`
   en valeurs de l'API ; `docs/api-external.md` (ligne neuve, `:1047`/`:1048` corrigées, validation fournisseur) vrai au code ; le
   CHANGELOG (avertissement en tête, changement de contrat écrit comme tel).
6. **Les manuels contre le code** (`admin-manual.tex`, `user-manual.tex`, les points d'AC13) ; contrôle aussi les **PDF** aplatis. Chaque
   phrase est-elle vraie ? En particulier : « la page ne nomme que le premier champ fautif » ; les deux voies de correction ; la liste
   « Trois cas échappent encore ».
7. **Les écrans** : fermeture du dialogue de validation sur ce code (`invoices/[id]/+page.svelte`), message tel quel ; côté fournisseur
   `err.message`. Le Vitest prouve-t-il la fermeture, ou passe-t-il à vide ?
8. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`mutations.log`, `mut/`) ? AC6 : l'unique attendu
   changé est-il bien `one_account_for_two_roles_is_named_once`, et aucun autre test existant n'a-t-il vu son attendu retouché
   (`git diff 049b0466 df8ea143 -- crates/*/tests/*`) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `check_written`, la requête verrouillée, `LockedDesignatedAccount`, la constante, `validate_account_of` et ses messages,
la nouvelle variante `DbError` et son rendu, les appels dans `invoices.rs` et `supplier_invoices.rs`, l'écran. Cherche : un ordre de refus qui
change un code d'erreur existant ; un `match` qui oublie un rôle ; un tri instable ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Compte retypé **et** archivé ; retypé et non imputable ; deux comptes retypés de types attendus opposés ; un compte désigné pour deux
rôles ; facture sans TVA (TVA due non écrite) ; facture fournisseur sans TVA ; compte désigné supprimé ; réglage absent (`NULL`) ; avoir
sur une facture dont la créance est retypée ; règlement d'une facture validée avant le retypage (hors périmètre ? écrit ?) ; retypage
concurrent d'une validation (le verrou partagé suffit-il ?). Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée
par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC14 : livré, testé, au lieu prescrit ? L'étape 0 jouée et ses recensements (38 `SET`, 17 `INSERT`, 14 `UPDATE`)
écrits au Change Log avec un verdict par site ? Les mutations nommées jouées et tuées ? Les gates au dernier commit de code
(journaux) ? Les écarts déclarés (gate complet seulement au dernier commit malgré l'exception `kesh-db` ; `cat >>` contre la consigne
Edit) ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e353-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque rapport :
findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou
preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
