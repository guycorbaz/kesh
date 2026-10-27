# Prompt — revue de code P2 de la Story 32-9b, passe ciblée « regression hunter », Sonnet

Tu relis **une seule remédiation** : celle de la revue de code P1 de la **partie b de la Story 32-9** de Kesh
(comptabilité suisse, Rust/Axum, MariaDB) — les outils MCP de lecture des contacts (`kesh_contacts`, `kesh_contact`,
`contacts:read`) et des pièces (`kesh_pieces`, `documents:read`, métadonnées seules). C'est une **passe ciblée**
(`CLAUDE.md`, § *La passe ciblée*) : une seule lentille, *regression hunter*, braquée sur le **seul commit de la
remédiation**. Le motif est mesuré sur ce dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire* — sur les
Epics 22 et 23, presque chaque passe tardive a trouvé un défaut **introduit par le patch précédent**, non un défaut
d'origine. Ta question n'est donc pas « la story est-elle bonne ? », mais **« ce commit a-t-il cassé, affaibli ou
mal dit quelque chose ? »**.

Worktree `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9b`. Le commit à relire :
`git -C /home/gcorbaz/devel/kesh-32-9 show 77452417` (« Story 32-9b : remédiation de la revue de code P1 »). Le commit
qui le suit (Change Log, Dev Agent Record, ce prompt) ne porte que des comptes rendus. Langue du rapport : français.
Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure.

Lis, pour le contexte : les trois rapports de la P1, `/home/gcorbaz/devel/kesh-gate-logs/329b-review-p1-{B,E,A}.md` ;
l'entrée « Revue de code P1 (32-9b) » du Change Log de la fiche
`_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md`, le Dev Agent Record de la partie b (sous-section
« revue de code P1 »), les cellules annotées de l'inventaire 32-9b et du tableau des largeurs, et le § *Ce que les
gardes ne voient pas* ; les choix **C-32-9-64 à C-32-9-67** (et l'annotation de C-32-9-60) du registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md`. Le Change Log déclare des gates, des mutations
(Mp1 à Mp9) et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/329b-*-p1*` et `329b-mutation-p1-Mp*.out` (lecture seule).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `2>`, `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier
  créé par erreur se **déclare** au rapport ; il ne se supprime pas — **aucun `rm`**, sous aucune forme (supprimer,
  c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `merge`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq`, ni `bc` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` et `cargo tree` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_329_*`, `P329*`, et les exemples publics
  SIX/ISO `CH9300762011623852957`, `RF18539007547034`). Ne lis rien sous `~/kesh-test/`.

## La lentille — regression hunter, sur `77452417` seulement

Pour **chaque** hunk du commit, demande-toi ce qu'il a pu casser, affaiblir ou faire dire de faux. En particulier :

1. **La borne de `search`** (`mcp/outils/commun.rs` : `RECHERCHE_MAX`, `recherche_bornee` ; `mcp/outils/contacts.rs`)
   — la borne compte-t-elle bien les caractères **rognés**, comme `services/factures.rs` (`recherche`) ? La recherche
   transmise au service est-elle inchangée (non rognée par l'outil) ? Le résumé (`recherche_fournie`) dit-il encore la
   seule présence, et pour une recherche **blanche** (`"   "`) dit-il la même chose qu'avant ? Le refus rend-il un
   texte fixe sans recopier la recherche ? `"maxLength":100` au schéma et la constante disent-ils la même chose
   (100 caractères ; un client JSON Schema compte des points de code, non des caractères rognés : l'écart est-il
   écrit ?) ? Le service du carnet et l'écran sont-ils bien restés sans borne (C-32-9-64) ?
2. **Le seed** (`kesh-seed/src/pieges.rs` : `byte_size` 4 101 et 4 202) — un autre test lisait-il la taille de P1 ou
   de P2, ou la taille d'une réponse de `kesh_pieces` sur la société piégée (`tailles_mesurees`, `plafond_au_maximum`,
   `pieces_sans_donnees_du_qr`, `services_rollout.rs`, l'E2E Playwright) ?
3. **`projections_des_contacts_et_des_pieces`** (`tests/mcp_enveloppe_e2e.rs`) — peut-il passer **à vide** ou par
   construction ? L'attendu est-il tiré des valeurs **posées** (pièges, valeurs littérales) et non d'une projection
   parallèle ? Ce qui est relu en base (adresse composée, instant d'import, taille) pourrait-il masquer une erreur de la
   sortie (une valeur fausse en base et dans la sortie à la fois) ? Les témoins de distinction (tailles contre
   identifiants, drapeaux croisés de C et Fo) suffisent-ils à tuer chaque transposition **voisine** de
   `contact_sortie` et de `piece_sortie` — `prenom` ↔ `nomDeFamille`, `ide` ↔ `numeroClient`, `client` ↔ `actif`,
   `id` ↔ `factureFournisseurId` de P1, `nomFichier` ↔ `typeMime` de P1 ? Raisonne sur les valeurs.
4. **L'exemption de la ligne `adresse`** (`INVENTAIRE`) — `inventaire_couvre_le_registre` l'accepte-t-il (piège ou
   exemption, jeux vides) ? Un test qui comptait les lignes à piège, ou les lignes `kesh_contacts`, change-t-il de
   sens ? L'exemption nomme-t-elle un test qui existe et lit bien la valeur exacte et son enveloppe ?
5. **`qr_d_une_piece_completee_lu_par_la_facture_fournisseur`** — la pièce posée respecte-t-elle la contrainte « tout
   ou rien » (`20261015000001_imported_documents_without_qr.sql:43-50`) ? La complétion passe-t-elle par la route
   réelle (`complete_import_once`) et ses contrôles (devise, routage IBAN, SCOR, égalité du montant) ? Le témoin final
   (`vus_ff`) prouve-t-il que la boucle a lu la facture créée **sous la clé entière** ? Les marques `*_QR_329B`
   pourraient-elles être filtrées par autre chose que l'absence de recopie (enveloppe, échappement JSON, limite de
   taille) ? La phrase du manuel « ne sont rendus par aucun outil » est-elle prouvée pour **tous** les outils, ou
   seulement pour ceux que la boucle appelle (un outil absent de `TEMOINS`, un jeu qui ne lit pas la facture créée) ?
6. **`filtres_de_kesh_contacts_transmis`** (`tests/mcp_outils_e2e.rs`) — l'ordre par nom (`AAA archivé 329b` avant
   `PIEGE_329_A07…`, avant `PIEGE_329_A08…`) dépend-il de la collation, et tient-il ? Les `created_at` posés par
   `UPDATE` sont-ils lus par le tri (`ContactSortBy::CreatedAt`) ? La recherche par l'IDE `P329A26` peut-elle rendre
   un autre contact par le plein texte du nom ? `sortDirection` sans `sortBy` est-il éprouvé, et doit-il l'être ?
7. **`recherche_de_kesh_contacts_bornee`** — 100 caractères `é` entourés d'espaces : le test prouve-t-il qu'ils sont
   **servis** (et non refusés pour une autre raison) ? Le refus à 101 est-il comparé à la réponse entière ?
8. **`mcp_sans_fichier`** (motif élargi : `\blister_tout\b|\bImportedSupplierInvoice\b`) — greppe toi-même le motif
   sur `crates/kesh-api/src/mcp/` : rend-il un **faux positif** sur une ligne de code réelle (doc-comment compris, si
   `est_commentaire` ne l'écarte pas) ? Les témoins rouges et verts testent-ils chaque branche neuve ? Les angles morts
   écrits (alias de crate, `ServeFile`) sont-ils exacts (`grep -nF "ServeFile" crates/kesh-api/src/mcp/mod.rs`) ?
9. **Le manuel** (`docs/manual/fr/admin-manual.tex`, § « Les contacts et les pièces ») **et son PDF aplati**, et le
   `CHANGELOG.md` — chaque phrase neuve contre le code : l'énumération de ce que rend `kesh_contacts` égale-t-elle
   `ContactSortie` (`mcp/outils/contacts.rs`) ? « la référence (QRR ou SCOR) et le montant … recopiés » contre
   `routes/imported_supplier_invoices.rs` (`complete_import_once`) ; « l'IBAN (ou le QR-IBAN) sous `contacts:read`
   seulement » contre `mcp/outils/factures_fournisseur.rs` (`Tiers::selon`) ; « le nom et l'adresse du créancier, le
   message et les informations de facturation … ne sont rendus par aucun outil » contre tous les outils du registre
   (aucun ne lit le journal d'audit, où `creditor_name` est inscrit à l'import et à l'écart ?). Une autre phrase du
   manuel, du `CHANGELOG.md`, d'une description d'outil ou d'un doc-comment dit-elle désormais le contraire ?
10. **Les mutations Mp1 à Mp9** (`329b-mutations-p1.log`, `329b-mutation-p1-Mp*.out`) : chacune est-elle tuée par le
    **test** visé et pour la **raison** visée (l'assertion relevée) ? Une mutation de la remédiation **non jouée**
    survivrait-elle — `sort_direction` ignoré, `is_supplier: None`, `recherche_bornee` qui compterait sans rogner,
    `actif: true` en dur ? Raisonne sur le code, n'exécute rien.
11. **Propagation** : greppe les symptômes corrigés (`ce qui a été lu`, `rien de ce qui`, `byte_size: 1`,
    `byte_size: 2`, `"search":{"type":"string"}`, `client_rue`, `41-48`) sur le dépôt : reste-t-il un site non traité,
    ou une entrée datée réécrite (un Change Log historique ne se réécrit pas ; une annotation datée s'ajoute) ?

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/329b-review-p2.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée), la correction proposée, et s'il est **né de la remédiation**
   (`77452417`) ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** (les onze ci-dessus) — sans cette liste, un
   « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
