# Prompt — validation P5 (ciblée) de la fiche v017-3b (#687 : une facture fournisseur au centime), une lentille, Sonnet

Tu valides la **remédiation de la passe P4** de la fiche de la Story v017-3b de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur le
**seul diff** du commit `docs(v017-3b) : remédiation de la validation P4 (refs #687)`. Modèle : **Sonnet** (passe
ciblée, non première passe). **Contexte frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la
fiche dit d'elle-même. Langue du rapport : **français**.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`.
Historique : P1 0/0/11/14 ; P2 0/0/11/17 (simplification C-v017-3-24) ; P3 0/0/1/18 (C-v017-3-26) ; P4 (Opus, ciblée)
0/0/3/7, trois MEDIUM nés de la remédiation P3. À la P4, l'orchestrateur a décidé **C-v017-3-27** : seule la **fiche** de
la facture passe aux décimales exactes (TTC, lignes, total du sélecteur de document) ; la **liste** garde son format.

## À lire

1. Le diff ciblé : `git log --oneline c5101b07..HEAD`, puis `git show <commit de la remédiation P4>` (fiche
   `v017-3b-facture-fournisseur-au-centime.md` et registre `v017-3-choix-autonomes.md`).
2. La fiche entière **après** le diff.
3. Le rapport P4, **après** ton examen : `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p4.md`.

## La lentille

### Axe obligatoire — chaque prémisse introduite par la remédiation P4, vérifiée **au code**

Pour **chaque** `fichier:ligne`, nom de fichier, de fonction, de constante ou de test que le diff introduit, ouvre la
source et cite la ligne lue. Au minimum :
- les bancs `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-settlement-page.test.ts` et
  `supplier-document-page.test.ts` : existent-ils, rendent-ils bien `./+page.svelte`, et peuvent-ils porter les cas
  prescrits (une facture `100.0040` annulée et ouverte ; le sélecteur ouvert sur une facture sans document) avec leurs
  mocks actuels ? Le sélecteur est-il atteignable dans `supplier-document-page.test.ts` (`canManage`, `sourceDocument`
  nul) ?
- `[id]/+page.svelte:386`, `:454-460`, `:501`, `:503` ; `AttachDocumentPicker.svelte:155`, `:164` ;
  `supplier-invoices/+page.svelte:93` ; `supplier-invoice-helpers.ts:11-14` ; `journal-entries/balance.ts:93-96` ;
- `docs/api-external.md:717` (`QR_AMOUNT_OUT_OF_RANGE` « à plus de deux décimales ») et `:733-745` ;
- `crates/kesh-api/src/routes/reconciliation.rs:1240-1258` (`masking_full`) et
  `crates/kesh-db/src/repositories/reconciliation.rs:907` (`SUPPLIER_MASKING_LIMIT`) ;
- `crates/kesh-core/src/accounting/mod.rs:18-22` (la ré-exportation prescrite tient-elle ?) ;
- le montage SQL par variables de session : `supplier_invoices` porte-t-il `supplier_invoice_number` et
  `purchase_journal_entry_id` ? le texte est-il jouable tel quel au client `mariadb` ?
- #830 (`gh issue view 830`).

### Ce que la remédiation a pu introduire

- C-v017-3-27 est-il appliqué partout — AC-10, AC-14, M16, L4, issues à ouvrir, registre — et la liste est-elle bien
  **retirée** de tout test, capture et mutation ? Une phrase restée en place dit-elle encore que la liste passe aux
  quatre décimales sans être marquée remplacée ?
- Les annotations « remplacé par » (C4) : chacune est-elle placée là où elle doit l'être, sans réécrire le texte
  d'origine ? Une phrase contredite reste-t-elle **sans** marque (cherche `quel que soit`, `annulée comprise`, `liste`,
  `automati`, `tableau des codes`, `corrigez-la`) ?
- Les amendements du registre sont-ils placés **sous** l'entrée qu'ils amendent ?
- La propagation : le symptôme corrigé a-t-il été grepé ailleurs (fiche, registre, prompts) ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p5.md`. Il
contient :

1. **Les findings**, numérotés `C1`, `C2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
