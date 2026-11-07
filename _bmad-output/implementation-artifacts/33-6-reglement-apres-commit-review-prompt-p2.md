# Prompt — revue de code P2 ciblée de la Story 33-6 : la remédiation de la P1, une lentille, contexte frais

Tu relis la **remédiation de la revue P1 de la Story 33-6** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : le règlement après son commit (#450), et **le rejeu d'un règlement sur réponse perdue**, rendu P1 à la lettre du barème (une écriture en trop, sans signal). Ce n'est pas la story entière : ce que la remédiation a écrit, et ce qu'elle a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff aa282c59 80cf06cb`** (commits `737e4c57`, `80cf06cb`). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e336-review-p2.md`, et rien d'autre.

Lis d'abord : les trois rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/e336-review-p1-{B,E,A}.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-6-reglement-apres-commit.md`. Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-6-p2.log`, `mut/33-6-p2-*.log` (`mut-33-6-p2.py` est à lire, jamais à exécuter), `33-6-p1/`, `33-6-p1b/`, `e2e-r33-6-p1b/`, `user-manual-33-6-p1.txt`, `admin-manual-33-6-p1.txt`. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-6-p1-*.png`.

⚠️ **Hors périmètre** : le bandeau « Base de données temporairement indisponible », visible sur une capture ; l'orchestrateur en ouvre l'issue.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`rebase`/`push`/`tag`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`,
  `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`** ; aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` nu est une fonction),
  `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `head`, `tail`, `cut`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, les tubes entre ces commandes,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires

1. **Le rejeu** : après une issue inconnue, reste-t-il un chemin vers un second règlement sans relecture réussie, sur la fiche et sur l'échéancier ? (dialogue déjà ouvert, deuxième onglet, `paidAt` périmé, `load()` périmé par `loadSeq`, filtres, pagination, un autre geste de la fiche.) Le montant proposé après relecture vient-il du reste relu ? Un `isApiError` faux couvre-t-il un défaut de programmation à tort (une exception levée après un succès : `loadSettlements`, `notifySuccess`) ? L'avis disparaît-il à tort ?
2. **`isUnknownOutcome`** : tous les appelants de `settleInvoice` sont-ils couverts ? Le côté fournisseur : `paySupplierInvoice` est refusé par statut (`supplier_invoices.rs`, `inv.status != "open"`) : la conclusion « pas de double écriture » tient-elle ?
3. **`McpSendDetail`** : `charger` avec `avant`, `saisie`, `edition` ; le premier chargement ; un 404 sur rechargement ; un texte non repris puis une décision ; la réponse vide.
4. **La garde par `syn`** (`invoice_settlement_response_e2e.rs`) : ce qu'elle ne voit pas (une attente dans une macro, un `.await` dans une fonction appelée, un autre nom de méthode `commit`) ; sa sonde exerce-t-elle `around` ? Chaque mutation `R-*` rougit-elle pour la bonne raison ?
5. **Les manuels, `.tex` et PDF aplatis, et le CHANGELOG** contre le code : `admin-manual.tex` § des envois (l'item de #768 récrit, « à une exception près ») ; `user-manual.tex`, « Si la réponse se perd » ; le CHANGELOG, qui ne doit plus promettre plus que le correctif.
6. **Les quatre locales** et les gardes recomptées (`i18n-keys.test.ts`, `i18n-repli-divergent-actif.test.ts`, `mcp-sends-i18n.test.ts`).
7. **La spec E2E** `invoices-settle-unknown-outcome.spec.ts` : la coupure après l'écriture est-elle réelle ? Passe-t-elle à vide ?

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
