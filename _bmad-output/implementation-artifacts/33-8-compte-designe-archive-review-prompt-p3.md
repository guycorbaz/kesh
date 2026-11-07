# Prompt — revue de code P3 ciblée de la Story 33-8 : la remédiation de la P2, une lentille, contexte frais

Tu relis la **remédiation de la revue P2 de la Story 33-8** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : l'archivage d'un compte désigné dans les paramètres de facturation (#486). La P2 avait trouvé que la liste des effets « réglage par réglage » du dialogue était fausse pour la seconde fois (un HIGH, un MEDIUM). La remédiation **retire la liste** et dit une règle générale. Ce n'est pas la story entière : ce que le commit de remédiation a écrit, et ce qu'il a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff e3fd7522 28566e24`** (deux commits de code, `b8d4dc6a`, puis `28566e24` qui borne la phrase du dialogue au règlement d'une pièce déjà validée). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p3.md`, et rien d'autre.

Lis d'abord : le rapport `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p2.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-8-compte-designe-archive.md` (branche `story/33-8-compte-designe-archive`). Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-8-p3.log`, `mut/33-8-p3-*.log` (`mut-33-8-p3.sh` est à lire, jamais à exécuter), `33-8-p2/`, `e2e-r33-8-p2/`, `user-manual-33-8-p2.txt`, `admin-manual-33-8-p2.txt`. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-8-*.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

⚠️ **Hors périmètre, déjà tracé** : les autres désignateurs de comptes (#939) ; le réarchivage, la course avec le `PUT` des réglages et le focus de l'avertissement (#954).

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

1. **La règle générale, contre le code** : « tant qu'il est archivé, toute écriture qui l'emploie est refusée, y compris le règlement d'une pièce déjà validée sur ce compte : pour celle-ci, il faut le réactiver, désigner un autre compte n'y changeant rien ». Tient-elle pour **chacun** des dix réglages, y compris les comptes d'écart, le produit par défaut et la TVA due, lus dans les réglages au moment de l'écriture (un autre compte désigné cesse-t-il bien alors de les faire employer, sans que la phrase dise le contraire) ? L'avoir et le solde d'une facture client sur une créance archivée ? Lis `journal_entries.rs` (garde `active`), `invoice_settlements_write.rs`, `credit_notes.rs`, `supplier_invoices.rs`, `company_invoice_settings.rs` (`*_for_write`). Cherche le réglage pour lequel la phrase dit faux.
2. **L'exception `vatDecompte`** : aucun lecteur hors des réglages, de leur route et de l'export ? (`git grep default_vat_decompte_account_id`.)
3. **Le dialogue** (`accounts/+page.svelte`) : la liste des libellés seuls, la règle une fois, la phrase du décompte à part ; le repli Svelte égal au catalogue `fr-CH` ; les quatre locales (justesse de `de`, `it`, `en`) ; plus aucune clé `accounts-archive-effect-*` orpheline ni manquante (`git grep`) ; `sitesTotal` 2850 et la raison du −7.
4. **Les tests neufs** (`accounts_designated_archive_e2e.rs`, `…until_reactivated`) : prouvent-ils ce que dit la règle (variante typée, autre compte désigné, réactivation, règlement) ? Passent-ils à vide ? La mutation `M-active` les fait-elle rougir pour la bonne raison ?
5. **Manuels, `.tex` et PDF aplatis, et CHANGELOG** : `user-manual.tex` § `sec:plan-comptable-archivage` (« pour une facture client, son solde et son avoir aussi ») contre `credit_notes.rs` et le solde ; `admin-manual.tex`, l'exception du compte créanciers absent du corps (L5) contre `routes/company_invoice_settings.rs` ; le CHANGELOG.
6. **Le doc-comment de `validate_account`** (seule ligne de production touchée en Rust, un commentaire) : dit-il vrai ?

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
