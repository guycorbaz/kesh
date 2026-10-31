# Prompt — validation P3 de la fiche v017-3b (#687 : une facture fournisseur au centime), deux lentilles, Sonnet

Tu valides la **fiche** de la Story v017-3b de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). **#687
(P3)** : une facture fournisseur peut porter un total TTC à plus de deux décimales (`line_total = quantité × prix` non
arrondi, aucune borne d'échelle) ; son règlement écrit une ligne de banque sous le centime, que la réconciliation ne
propose jamais. La fiche retient de **refuser à la création** une ligne dont le montant n'est pas au centime (fonction
pure unique, deux routes de création, borne de quatre décimales sur quantité et prix) et, pour les factures
**existantes**, de ne rien modifier : la fiche de la facture porte un avertissement statique qui renvoie au manuel.
Aucune migration, aucune donnée réécrite.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`. Langue du
rapport : **français**. Modèle : **Sonnet**. Passe **complète** (troisième passe de validation : la simplification de
la P2 réécrit beaucoup). **Contexte frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la fiche
dit d'elle-même.

Historique : P1 (Sonnet) 0/0/11/14, P2 (Opus) 0/0/11/17. À la P2, l'orchestrateur a décidé une **simplification**
(C-v017-3-24) : la matrice de conduite par état de l'AC-10 est **retirée** ; la fiche porte désormais un avertissement
unique et statique, et le manuel une sous-section `sec:facture-fournisseur-sous-centime` qui décrit les voies en prose.
Le commit de cette remédiation est `docs(v017-3b) : remédiation P2, avertissement simplifié` (`git log --oneline
c5101b07..HEAD`, `git show <commit>`). Rapports antérieurs, à lire **après** ton propre examen, pour les axes obligatoires
seulement : `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p2-R.md` et `-F.md`.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-3b-facture-fournisseur-au-centime.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-3-choix-autonomes.md` (C-v017-3-1, 13 à 20, 24).
3. Les issues, **en lecture seule** : `gh issue view 687 --json title,body,comments`, idem 686, 476.
4. La contrainte du Project Lead : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
5. Le code : `crates/kesh-db/src/repositories/supplier_invoices.rs` (`create_in_tx`, `generate_purchase_journal_lines`,
   `pay_in_tx_as`, `cancel_in_tx`, `cancel_settlement_in_tx`, `supplier_settlement_cancel_blocker*`) ;
   `crates/kesh-core/src/accounting/vat.rs` ; `crates/kesh-api/src/routes/supplier_invoices.rs`
   (`to_new_supplier_invoice`), `imported_supplier_invoices.rs`, `invoices.rs` (bornes d'échelle), `limits.rs`,
   `errors.rs` ; `crates/kesh-db/src/repositories/reconciliation.rs` (propositions fournisseur) ;
   `crates/kesh-payment/src/pain001/mod.rs` ; `crates/kesh-api/tests/inbox_import_e2e.rs`,
   `crates/kesh-db/tests/reconciliation_repository.rs` ; `frontend/src/lib/features/supplier-invoices/`
   (`supplier-invoice-form.ts`, `SupplierInvoiceForm.svelte`) et
   `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` ; `crates/kesh-seed`.
6. Le **manuel** : `docs/manual/fr/user-manual.tex`, § `sec:factures-fournisseurs` (l. 1563-1720 environ) et
   `sec:matching-fournisseur` (l. 2304) ; le PDF aplati par
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, **vers la sortie standard seulement**.
7. La fiche sœur `v017-3a-rattacher-ecriture-existante.md` (le lien fonctionnel, L5) et, pour les collisions seulement,
   `/home/gcorbaz/devel/kesh-0162/_bmad-output/implementation-artifacts/v016-2-releve-manuel.md` (lecture seule).
8. `CLAUDE.md` du worktree et `docs/agents-orchestration.md`.

## Deux lentilles, en contexte frais chacune

### Axes obligatoires des deux lentilles

1. **Rien de la matrice ne doit survivre** — dans les AC, les décisions, les mutations, les tests prescrits (Vitest, HTTP,
   E2E), les captures, le manuel prescrit, les sections *Dépendance* et *Collision*, et le registre (hors entrées et
   amendements datés, qui sont l'historique). Cherche : `matrice`, `cancellable`, `settlementCancellable`,
   `lastConfirmedBatch`, `settlementType`, `canManage`, `selon l'état`, `conduite`, `si un bouton`, « levez-la »,
   « Annulez-la » et toute condition d'affichage autre que « le total n'est pas au centime ». Une phrase du manuel prescrit
   qui décrit un comportement de l'écran autre que l'avertissement statique est un finding.
2. **La remédiation P2** : chaque finding P2 appliqué, absorbé par la simplification ou rendu sans objet, comme le Change
   Log l'annonce ? La remédiation a-t-elle introduit un défaut — `purchase_lines_form` (ordre positivité → centime →
   TVA, juste au code ? la complétion peut-elle l'appeler avant `create_in_tx` sans dupliquer ni déplacer un refus
   existant ?), cas négatif de M10, prédicat de centime hors de `linesInvalid`, contrat de `hasSubCentime`, clés i18n
   nommées, méthode de recompte de `CLES_RELEVEES`, sous-section du manuel (chaque voie est-elle vraie au code, ou
   marquée « à établir à l'étape 0 » ?) ? Le symptôme corrigé a-t-il été grepé ailleurs (propagation) ?

### Lentille R — la règle et sa preuve

- **Les prémisses** : chaque `fichier:ligne` cité dit-il ce que la fiche lui fait dire ?
- **La règle (AC-1)** : « `quantité × prix` au centime après `normalize()` » suffit-elle à garantir un TTC au centime
  (TVA arrondie par ligne — relis `line_vat_amount` et `Money::round_to_centimes`) ? Les cas de test sont-ils justes ?
  La borne de quatre décimales est-elle compatible avec les colonnes et avec ce que le formulaire admet ?
- **Les points d'entrée** : `create_in_tx` est-il le **seul** créateur de `supplier_invoices` (cherche tout `INSERT INTO
  supplier_invoices` et tout appelant, seed et tests compris) ? Un autre chemin écrit-il `total_amount` ou des lignes
  (modification, import d'installation, restauration) ? La restauration d'une sauvegarde 0.16.x qui porte un TTC sous le
  centime passe-t-elle encore (elle **doit** passer : données à garder) ?
- **L'ordre des refus** de la complétion (AC-6) et de `create_in_tx` (AC-3) : est-il cohérent avec le code, et le
  refus n'écrit-il rien ?
- **L'existant (AC-8, AC-9, AC-11, AC-19)** : rien n'est réécrit ; le montage de la fixture est-il exécutable et
  équilibré ; la voie nominale de l'AC-11 est-elle jouable au code ; la restauration de l'AC-19 tient-elle ?
- **Les tests existants (AC-7)** : lesquels cassent ? Recense-les (cherche des produits quantité × prix sous le
  centime dans `crates/*/tests`, `crates/kesh-seed`, `frontend/tests/e2e`, `frontend/src/**/*.test.ts`).
- **Les mutations (M1 à M14)** : chacune rougit-elle le test annoncé ?

### Lentille F — l'écran, les textes et ce qu'ils affirment

- **Le manuel (AC-16)** — axe obligatoire, au `.tex` **et** au PDF aplati : chaque phrase devenue fausse est-elle
  nommée (« deux décimales », « sous-centime », « centime-exact », la saisie d'une ligne, la complétion d'une pièce) ?
  La sous-section `sec:facture-fournisseur-sous-centime` (voies 1 à 8) dit-elle vrai au code, voie par voie, ou
  marque-t-elle « à établir à l'étape 0 » ce qui ne l'est pas ? Tous les renvois pointent-ils ce label ?
- **L'écran (AC-10, AC-12 à AC-15)** : l'avertissement statique dit-il vrai **quel que soit** l'état (annulée comprise)
  et le rôle, sans promettre de geste ? Le message de ligne et le message au clic sont-ils cohérents avec le refus
  serveur, et leur ordre avec `submit()` ? L'avis existant `supplier-invoices-subcentime` reste-t-il juste ? Les captures
  couvrent-elles chaque contrôle visible touché ?
- **i18n** : clés dans les quatre catalogues, repli d'`errors.rs` égal au catalogue fr-CH.
- **`docs/api-external.md`, CHANGELOG (AC-17, AC-18)** : le changement de contrat est-il dit ? Aucune version bumpée ?
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; données réelles absentes ;
  défaut voisin (pain.001) renvoyé à une issue et non intégré ; « aucune donnée existante modifiée d'office » tenu
  partout.
- **Les décomptes et numéros de ligne** : recompte-les.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p3-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`… ou `F1`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu,
   **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de présence ou
   d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
