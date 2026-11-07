# Prompt — revue de code P3 ciblée de la Story 33-6 : la remédiation de la P2, une lentille, contexte frais

Tu relis la **remédiation de la revue P2 de la Story 33-6** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : le règlement après son commit (#450) et le rejeu sur réponse perdue. Ce n'est pas la story entière : ce que le commit de remédiation a écrit, et ce qu'il a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff d17b3439 242b9191`** (un commit de code, `242b9191`). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e336-review-p3.md`, et rien d'autre.

Lis d'abord : le rapport `/home/gcorbaz/devel/kesh-gate-logs/e336-review-p2.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-6-reglement-apres-commit.md` (branche `story/33-6-reglement-apres-commit`). Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-6-p3.log`, `mut/33-6-p3-*.log` (`mut-33-6-p3.sh` est à lire, jamais à exécuter), `33-6-p2/`, `e2e-r33-6-p2/`. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-6-p1-*.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

⚠️ **Hors périmètre** : l'idempotence côté serveur (#951), déjà tracée ; le 404 au rechargement de `McpSendDetail` (P2-6, premier angle), dont l'issue est demandée.

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

1. **`isSettlementOutcomeUnknown`** (`unknown-outcome.ts`) : tout 5xx et toute erreur hors `ApiError` valent-ils issue inconnue, aux deux écrans ? `isUnknownOutcome` est-il inchangé pour ses autres appelants (sauvegarde, restauration, envois MCP), et son doc-comment dit-il vrai ? Une erreur du client qui n'est ni 4xx ni 5xx (statut 3xx, statut 0 hors `TIMEOUT`/`NETWORK_ERROR`) laisse-t-elle le dialogue ouvert à tort ? Le `else` retiré des deux écrans laissait-il un cas atteignable ? Un 4xx, refus certain, garde-t-il son comportement ?
2. **La phrase de l'avis**, reformulée aux quatre locales et au repli (« le serveur ne l'a pas confirmé ») : juste pour chacun des cas (réseau, délai, 5xx, réponse illisible) ? Justesse de `de`, `it`, `en` ; repli égal au catalogue `fr-CH` ; les autres sites de « la réponse n'est pas parvenue » (`AutoBackupSection`, envois MCP) n'ont pas à bouger.
3. **La garde `syn`** (`invoice_settlement_response_e2e.rs`) : le critère « l'`.await` de l'appel est le dernier du corps, hors `rollback` » ; l'ordre de visite de `syn` est-il bien l'ordre d'évaluation dans les formes réelles (`cancel_settlement` et son bloc `async`, `decider_envoi`) ? `read_before` peut-il devenir vrai à tort ou faux à tort ? L'exemption `rollback` ouvre-t-elle un trou (`tx.rollback().await` portant autre chose) ? Les limites écrites sont-elles exactes et complètes ? Chaque mutation `R-*`, dont `R-match`, rougit-elle pour la bonne raison ?
4. **P2-3** : la facture capturée avant l'envoi ; le chemin du succès et du refus restent-ils justes ?
5. **P2-6** : `relu.content !== saisie` et la remise à `null` après une décision ; un cas où l'utilisateur perdrait son texte sans le dire ?
6. **La spec E2E** du règlement partiel : le montant `100.00` est-il bien le reste relu ? Passe-t-elle à vide ? Le journal `e2e-r33-6-p2/` la montre-t-il verte, et les rouges sont-ils les échecs attendus (`docs/testing.md`) ?
7. **Manuels, `.tex` et PDF aplatis, et CHANGELOG** contre le code : `user-manual.tex`, « Si la réponse se perd » (panne du serveur, onglet, délai dépassé) ; le titre et la limite du CHANGELOG, le renvoi #951.
8. **Les gardes recomptées** : `sitesTotal` 2838, et la raison du −2.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
