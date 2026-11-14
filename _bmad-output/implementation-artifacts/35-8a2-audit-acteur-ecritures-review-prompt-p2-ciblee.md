# Prompt — revue de code P2 ciblée de la Story 35-8a2 (refs #431), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus, trois lentilles), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e358a2`, branche `story/35-8a2-audit-acteur-ecritures`. La P1 a rendu deux MEDIUM distincts — M-A
(B1 = A2 = E2 : manuels et CHANGELOG rangeaient le règlement d'une facture passé par un rapprochement à la fois du côté de la clé et
du côté du créateur) et M-B (A1 = E1 : aucun test d'exécution de l'écriture d'un rapprochement par clé, hors `POST
/reconciliation/manual`) — et des LOW (rapports `/home/gcorbaz/devel/kesh-gate-logs/e358a2-review-p1-{B,E,A}.md`). Déjà tracés, à ne
pas rapporter de nouveau : #1130 (None positionnel des fonctions exemptées), #1131 (annulation d'un rapprochement sans clé), #1132
(`pieges.rs`) ; et les LOW de la P1 non traités, que l'orchestrateur ouvre en issues : *à compléter par l'orchestrateur* (B5, les
cinq littéraux `Acteur { user_id, api_key_id: actor_api_key_id }` de `routes/reconciliation.rs`).

## Périmètre : le seul commit `ec8117bcc`

`git -C /home/gcorbaz/devel/kesh-e358a2 show ec8117bcc` — `CHANGELOG.md`, `docs/api-external.md`, `docs/manual/fr/{admin,user}-manual.tex`,
`crates/kesh-db/src/{retry.rs,repositories/journal_entries.rs,repositories/supplier_invoices.rs}` (commentaires seulement : à
vérifier), et six fichiers de tests de `crates/kesh-api/tests/` (`common/mod.rs`, `reconciliation_e2e.rs`,
`reconciliation_manual_e2e.rs`, `journal_entry_reversal_e2e.rs`, `invoice_unvalidate_e2e.rs`, `audit_acteur_garde.rs`). Compte rendu
et PDF : le commit suivant (entrée « Remédiation P1 » du Change Log de la fiche, annotations du Dev Agent Record). Journaux sous
`/home/gcorbaz/devel/kesh-gate-logs/e358a2/` : `p1-clippy.log`, `p1-cibles.log`, `p1-manuels-make.log`, `p1-mut/*.log`,
`p1-db-reset.log`, `p1-test-fast.log`.

## Ce que tu vérifies

1. **Les textes, geste par geste, contre le code** (M-A). La phrase des trois textes (`CHANGELOG.md`, entrées 35-8a1 et 35-8a2 ;
   item *Journal d'audit* du manuel d'administration, ses **deux** phrases ; tableau du journal d'audit du manuel utilisateur) range
   désormais : du côté de la clé, « toutes » les écritures d'un rapprochement, y compris le règlement d'une facture client et le
   paiement d'une facture fournisseur qu'il enregistre ; du côté du créateur, « un règlement saisi sur la facture »
   (`POST /invoices/{id}/settlements`), le solde de son reste, « paiement depuis la fiche ou par un lot de paiement ». Relis toi-même
   chaque chemin et cite la ligne : `accept_one_invoice` (`routes/reconciliation.rs`, `create_in_tx` avec `Acteur { …, api_key_id:
   actor_api_key_id }`) ; `accept_one_supplier_invoice` → `pay_in_tx_as(…, actor)` → `create_in_tx(…, actor, …)` ;
   `settle_invoice_handler` → `invoice_settlements_write::settle_invoice` (`Acteur::utilisateur`) ; `pay_supplier_invoice` →
   `supplier_invoices::pay` → `pay_in_tx` (`api_key_id: None`) ; `payment_batches::confirm_batch` → `pay_in_tx` ;
   `write_off_invoice_handler`. Un geste tombe-t-il encore sous deux règles, ou sous aucune ? « toutes celles qu'un rapprochement
   bancaire passe » est-il vrai pour **chaque** écriture que `POST /reconciliation/accept`, `/manual`, `/split` créent (y compris
   l'écart d'arrondi, s'il fait une écriture à part) ? Les traces voisines du même geste (`invoice.paid`, `supplier_invoice.paid`)
   sont-elles cohérentes avec la première phrase de l'item du manuel d'administration (« règlements saisis sur la facture », « paiement
   depuis la fiche ou par un lot ») ?
2. **Les PDF aplatis** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis `/usr/bin/grep -o`, de même pour
   le manuel utilisateur : le texte du PDF reprend-il le `.tex` du commit ? Les PDF sont-ils postérieurs au commit des `.tex` ?
3. **`docs/api-external.md`** (`:677`, `:713`, `:739`) : les trois ajouts disent-ils vrai ? En particulier, `journal_entry.created`
   « de l'écriture que l'acceptation passe, quel qu'en soit le type », et, pour l'annulation, `journal_entry.reversed` et le
   `journal_entry.created` de la contre-passation pour le seul genre `entry` (`reconciliation_cancel.rs`, bras `Entry` contre
   `InvoiceSettlement`/`SupplierSettlement`, ligne lue).
4. **Les tests neufs et étendus** (M-B) : `accept_letters_a_fully_settled_invoice`,
   `an_api_key_accepts_a_supplier_invoice_and_is_named_everywhere`, `an_api_key_rule_and_split_entries_are_attributed_to_the_key`
   (neuf), `an_api_key_cancels_and_is_named_in_the_audit`. Chaque assertion porte-t-elle sur la **bonne** écriture (l'encaissement et
   non la vente ; l'écriture rapprochée pour `journal_entry.reversed`, l'inverse pour `journal_entry.created`) ? L'attribution des
   écritures du test neuf à leur genre (`ruled` → `rule`, sinon `split`) peut-elle se tromper sans rougir ? Le `user_id` attendu
   (`ctx.user_id`, `c.seeded.admin_user_id`) est-il bien le créateur de la clé (la clé est créée par `ctx.jwt` / `c.jwt` : lis
   `setup_company` et `forge_jwt`) ? Un test peut-il passer à vide (proposition refusée, `accepted` vide) ? 
5. **`common::audit_actor_unique`** : unicité réellement assertée (`fetch_all`, longueur 1) ; `acteur_de`, le témoin d'AC4 et le
   test du rapprochement manuel (forme en ligne, ce binaire n'ayant pas `common`) l'emploient-ils tous ? Reste-t-il un `fetch_one` d'acteur
   ajouté par la story 35-8a2 (`git diff 4531669f8 ec8117bcc -- 'crates/kesh-api/tests/*'`) ?
6. **Les mutations** (`p1-mut/*.log`) : M10a (bras `split` d'`accept_one` sans clé), M10b (bras `rule`), M11
   (`let actor_api_key_id: Option<i64> = None;` dans `post_split_once`), M12 (`pay_in_tx_as` qui passe `Acteur::utilisateur(user_id)`
   à `create_in_tx`), M13 (bras `invoice` sans clé) : chacune tuée par le test annoncé, pour la bonne raison (message d'échec cité) ;
   restauration constatée (« 0 ligne ») ; la garde verte sous M10/M11 (angle mort) est-elle bien écrite au Change Log ? Propose une
   mutation plausible qui survivrait encore.
7. **Les commentaires de production** (`retry.rs`, `journal_entries.rs`, `supplier_invoices.rs`) : le diff ne change-t-il que des
   commentaires ? L'exemple de `retry_on_deadlock` est-il juste (signature de `journal_entries::create`, `Acteur` `Copy`) ?
8. **La garde** (`audit_acteur_garde.rs`) : seuls la doc du motif 5 et l'échantillon `g` de la sonde (h) changent-ils ? L'échantillon
   `g` reste-t-il non compté pour la bonne raison ?
9. **Le compte rendu et les gates** au commit `ec8117bcc` : test-fast `--ci` voie 2 au chiffre du journal, postérieur au commit ;
   clippy ; `gate-ci` et E2E déclarés « à la clôture », non affirmés ; entrées antérieures du Change Log et du Dev Agent Record
   annotées sans réécriture (`git show <commit du compte rendu>`, lignes `-`).

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport (outil Write). **Aucune redirection, sous aucune forme** — ni `>`, ni `>>`, ni
`tee`, ni `2>`, ni `2>&1`, ni `2>/dev/null`, ni `> /dev/null` —, **aucune substitution de processus** (`<(…)`), aucun fichier temporaire.
Plusieurs relecteurs de cette epic ont écrit `2>/dev/null` « par réflexe » : relis chaque commande avant de la lancer. Aucun `rm`, `pkill`,
`kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`,
`scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
`curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni
`~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`,
`find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`. ⚠️ `grep` est une fonction vers
`ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un
écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e358a2-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
