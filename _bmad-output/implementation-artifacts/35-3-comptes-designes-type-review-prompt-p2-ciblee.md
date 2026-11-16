# Prompt — revue de code P2 ciblée de la Story 35-3 (#537), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet**, en lecture seule. Worktree `/home/gcorbaz/devel/kesh-e353`, branche
`story/35-3-comptes-designes-type`.

## Périmètre : le seul commit `1ceae43f`

`git -C /home/gcorbaz/devel/kesh-e353 show 1ceae43f` — une phrase de la note *Rôles des comptes* du manuel utilisateur
(`docs/manual/fr/user-manual.tex`, ligne 493) et son PDF régénéré. Elle remédie trois findings de la P1 :
- **A1 (MEDIUM)** : « le compte de produit par défaut n'est pas contrôlé à l'usage » se lisait comme une exception au contrôle de type ;
  rapport `/home/gcorbaz/devel/kesh-gate-logs/e353-review-p1-A.md` § A1 ;
- **A2 (LOW)** : le solde du reste d'un règlement relit la TVA due sans en contrôler le type (#1044) ; même rapport § A2 ;
- **E4 (LOW)** : le règlement d'une facture déjà validée solde la créance sur le compte débité ; `e353-review-p1-E.md` § E4.

## Ce que tu vérifies, chaque affirmation de la phrase contre le code

1. « son type et son archivage restent contrôlés à la validation » (compte de produit par défaut) : `crates/kesh-db/src/repositories/invoices.rs`
   (`validate_line_revenue_accounts_in_tx`, la raison `Inactive`/`NotRevenue`/`NotPostable`, l'appel qui ajoute le site `None` du compte par
   défaut). Vrai pour **toute** facture validée ? Et l'archivage : « inactif » = archivé ?
2. « Le solde du reste d'un règlement relit lui aussi la TVA due dans les réglages sans en contrôler le type » : trouve le chemin
   (`vat_payable_account_for_write`, `company_invoice_settings.rs`, et son appelant côté règlement) ; « lui aussi » renvoie à l'avoir : la
   phrase précédente dit-elle bien que l'avoir relit la TVA due sans la contrôler ? #1044 est-elle le bon numéro (titre au registre de la
   fiche ou au CHANGELOG, sans `gh`) ?
3. « le règlement d'une facture déjà validée continue de solder la créance sur le compte que la facture a débité » : lis
   `invoice_settlements_write.rs` (et le rapprochement s'il passe ailleurs) : la créance est-elle lue sur l'écriture de vente, et non dans
   les réglages ? Si un chemin de règlement lit la créance dans les réglages, la phrase est fausse.
4. **Propagation** : la phrase contredit-elle un autre passage des manuels (`user-manual.tex` `:452-461`, `:1169`, `:1788-1796` ;
   `admin-manual.tex` `:3816-3822`) ou le CHANGELOG (`## [0.17.7]`, entrée #537) ? Grep `produit par défaut`, `solde du reste`, `1044`
   sur `docs/manual/fr/*.tex` et `CHANGELOG.md`.
5. **Le PDF** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` porte-t-il la phrase entière ? Le diff ne touche-t-il que
   `user-manual.{tex,pdf}` ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, aucune redirection (`>`, `>>`, `tee`, `2>`), aucun fichier temporaire. Aucun `rm`,
`pkill`, `kill`, aucun `git` qui écrit, aucun interpréteur ni script (`python3`, `node`, `bash -c`, `jq`, `scripts/*` dont
`scripts/prepare-release.sh`, `make`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`), aucune commande `gh`, aucun accès réseau.
Ne lis rien sous `~/kesh/`. Permis : `git show/diff/log/grep`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection, `cat`, `ls`, `find` sans
`-exec`, `wc`, `pdftotext … -`. ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de
présence ou d'absence s'appuie sur une sortie copiée au rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e353-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu, ligne
lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
