# Prompt — revue de code P2 ciblée de la Story 33-4, une lentille, sur la remédiation de la P1

Tu relis **la seule remédiation de la revue P1** de la Story 33-4 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11),
version 0.17.5, sans migration. Kesh tient des livres réels. Passe **ciblée**, contexte frais, langue du rapport : français. Modèle : Sonnet ou Opus
(la remédiation touche plusieurs modules et une règle de lot : Haiku n'est pas indiqué).

Worktree `/home/gcorbaz/devel/kesh-e334`, branche `story/33-4-bornes-annees`. Diff à relire :
`git -C /home/gcorbaz/devel/kesh-e334 diff 16675c3b HEAD`. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e334-review-p2.md`, et rien d'autre.
Findings de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e334-review-p1-{B,E,A}.md`.

## Ce que la remédiation prétend

1. **E-1 = B-3 = A-1** : un filtre de date n'interroge pas tant que la date est incomplète (`frontend/src/lib/shared/utils/date-filtre.ts`, seul helper) ; il garde sa dernière requête, sans notification ni liste vidée ; la sortie du champ restaure ; effacer retire le filtre. Écrans : `routes/(app)/invoices`, `invoices/due-dates`, `journal-entries`, `audit-log` ; `open-items` y prend son prédicat. Preuve navigateur : `frontend/tests/e2e/date-filtre-frappe.spec.ts`.
2. **B-1 = E-2 = A-2** : `POST /reconciliation/accept` : refus par proposition (`accept_one_split`), `200`, `failed[]`, `VALIDATION_ERROR`, `details.reason = value_date_out_of_range` ; `deserialize_with` retiré.
3. **B-2 = E-3 = A-3** : `docs/api-external.md` § 5 et le message de `reconciliation.rs:932`.
4. **LOW** : garde (`ParProposition`, `#[foo]`, preuve de classe A par nom de test, en-tête), témoin MCP, `mode=fiscal_year`, `?dateFrom=` vide, CAMT à travers HTTP, bornes de la saisie manuelle.

## Les axes obligatoires

1. **Le passage de `bind:value` à `value` + `onchange` (écritures, audit)** : `buildQuery`, l'export, le bouton de remise à zéro, l'URL, le rechargement, un autre filtre changé pendant une frappe ; un champ qui garde un affichage faux ; le retour arrière.
2. **`valeurDuChamp` et `badInput`** : un segment vidé, un champ vide par le bouton du navigateur, une date impossible (31.02) ; Firefox et Safari (raisonnement, écrit comme tel) ; le champ à filtre vide quitté sur une date incomplète.
3. **L'inventaire clos** (C-e334-38) : refais-le depuis le symptôme, `type="date"` et tout envoi déclenché par un `$effect`, `oninput`, `onchange` ; un filtre oublié ; un autre écran dont le serveur refuse désormais une année en cours de frappe (formulaires : le refus d'un clic est-il lisible ? 422 en texte, `parseErrorResponse`).
4. **Les dates de l'adresse** : `filtreDeLAdresse` et les deux `query-helpers` : un lien ancien ; `syncUrl` qui réécrit. ⚠️ Déclaré : aucun test de page pour `filtreDeLAdresse` aux sites factures et échéancier ; à juger.
5. **`accept`** : l'ordre du nouveau refus dans `accept_one_split` ; `SAVEPOINT` ; le pré-vol ; les autres types de proposition (`invoice`, `rule`, `supplier_invoice` n'ont pas de `valueDate` ?) ; la date venue de la transaction ; la garde (`AcceptProposalInput`).
6. **Le manuel** : `docs/manual/fr/user-manual.tex`, § *Recherche d'écritures*, et le **PDF** `docs/manual/fr/user-manual.pdf`, aplati (`pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`) ; `docs/api-external.md` § 5 et § *Accepter des propositions* contre le code, route par route ; `CHANGELOG.md`.
7. **Les captures**, thème clair : `/home/gcorbaz/devel/kesh-gate-logs/e334/captures/` (`*-frappe-0202.png`, `*-date-complete.png`, `*-sortie-restauree.png`).
8. **Les tests** : muets, ou à vide ; la spec fait-elle bien taper chiffre à chiffre ? La fiche, le Change Log, le registre (C-e334-37, 38) : décomptes recomptés aux journaux `kesh-gate-logs/e334/` : `gates-head.log`, `gate-test-fast.log`, `front-*.log`, `e2e-r6/`, `rouge-navigateur.log`, `mut/synthese-p1-front.log`, `mut/P1-*.log`. ⚠️ `mut/synthese-p1-vide-mauvaise-racine.log` est une campagne à vide, écartée : à ne pas compter.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se déclare.
- **Aucun `rm`**, `pkill`, `killall`. **Aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `stash`, `reset`, `rebase`, `push`, `tag`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`** ; aucun réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (`grep` nu est une fonction), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Le rapport

Sévérités CRITICAL, HIGH, MEDIUM, LOW, chaque finding avec fichier, ligne, extrait lu et remédiation ; décompte recompté. **Les axes réellement exercés et ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas. Aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`).
