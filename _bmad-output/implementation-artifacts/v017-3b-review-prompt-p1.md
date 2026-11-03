# Revue de code P1 — Story v017-3b (#687) : une facture fournisseur naît au centime

**Modèle : Sonnet**, contexte frais, trois lentilles indépendantes **B** (Blind), **E** (Edge cases), **A**
(Acceptance), une instance par lentille. Langue : français soigné.

## Périmètre

- Worktree : `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`.
- Le diff de la story : `git diff 899f564a..HEAD` (commits `a95ddae2` tests rouges, `bbb13896` implémentation, puis
  manuel, PDF et compte rendu). Un **diff aplati unique**, non la suite des commits.
- La fiche : `_bmad-output/implementation-artifacts/v017-3b-facture-fournisseur-au-centime.md` (AC-1 à AC-19, Dev
  Agent Record, Change Log) ; le registre `v017-3-choix-autonomes.md` (C-v017-3-13 à 35).
- Contrainte du Project Lead : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## ⛔ Lecture seule stricte

Aucune commande qui écrit, compile ou exécute : **ni** `cargo`, `npm`, `npx`, `make`, `latexmk`, `python3`, `node`,
`docker`, `curl`, `claude`, ni aucun script du dépôt (`scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`…), ni `sed -i`, `rm`, `pkill`, `killall`, ni `git checkout`, `switch`, `reset`, `stash`,
`commit`, `add` sous aucune forme. **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier
dans le scratchpad. Permis : `git diff`, `git show`, `git log`, `grep`, `rg`, `sed -n`, `cat`, `wc`, et
`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '` vers la sortie standard. Le seul fichier écrit est ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v017-3b-review-p1-{B,E,A}.md` (ta lettre). Chaque finding : sévérité
(CRITICAL/HIGH/MEDIUM/LOW), lieu (`fichier:ligne`), **ligne lue citée** (sortie `grep -nF` copiée), constat, remède.
Le rapport finit par **« Axes exercés »** et **« Axes non exercés »** — un « 0 finding » sans cette liste ne compte pas
— puis le décompte recompté depuis la liste.

## Axes (toutes lentilles, chacune selon son angle)

1. **La règle** : `kesh-core/src/accounting/purchase.rs` (normalisation, produit rendu, erreur normalisée) ;
   `purchase_lines_form` (`kesh-db/src/repositories/supplier_invoices.rs`) — ordre positivité → centime → TVA, rang
   1-based, aucun second produit dans `create_in_tx` ni dans la complétion.
2. **Les deux chemins** : `to_new_supplier_invoice` (bornes **avant** le `match` de paiement, normalisées ; écart voulu
   avec la facture client), la complétion (`imported_supplier_invoices.rs`, passe de forme avant `AMOUNT_MISMATCH`),
   la conversion unique `DbError::SupplierLineNotInCentimes` → `400` dans `errors.rs`, repli égal au catalogue (G9).
3. **Données existantes** (contrainte du 2026-10-30) : aucune migration, aucun `UPDATE`/`INSERT`/`DELETE` nouveau en
   production (`git diff c5101b07 -- 'crates/*/src' | grep -nE '^\+.*\b(UPDATE|INSERT|DELETE)\b'`) ; une facture
   existante sous le centime reste lisible, payable, annulable, **non réécrite** (AC-9) ; la restauration d'une
   sauvegarde la rend à l'identique sans passer par `create_in_tx` (AC-19, `admin_full_import_e2e.rs`). Le montage
   SQL de test (`crates/kesh-db/tests/support/montage-sous-centime.sql`) ne vit que sous `tests/`.
4. **L'écran** : `SupplierInvoiceForm.svelte` (prédicat après `linesInvalid`, message de ligne, message dédié au clic,
   refus serveur sous sa ligne, bouton non désactivé) ; le miroir `lineNotInCentimes` (`big.js`, virgule, quantité
   vide = 1, illisible → rien, `toFixed`) ; un seul `hasSubCentime` (`invoices/invoice-helpers.ts`, contrat `null`) ;
   `SupplierInvoiceTotal.svelte` (avertissement sauf annulée, texte exact, `formatExactAmount`) ; cellules de lignes
   et total du sélecteur exacts ; la liste inchangée (#831).
5. **i18n** : quatre clés dans les quatre catalogues, variables identiques ; replis du front égaux au catalogue fr-CH ;
   `CLES_RELEVEES` 344 → 348 et `sitesTotal` 2619 → 2623, recomptés depuis les sources.
6. **Le manuel et son PDF** : `docs/manual/fr/user-manual.tex` — paragraphe « Une ligne se saisit au centime »,
   `\subsubsection` `sec:facture-fournisseur-sous-centime`, renvois depuis `sec:matching-fournisseur`, les paiements
   pain.001, le diagnostic, la liste des refus de la complétion. Chaque affirmation de comportement contre le code
   (annulation du règlement puis de la facture, contre-passation d'un règlement détaché, réouverture, lot, détachement
   du document, masquage de la règle d'affectation). **Le PDF** `docs/manual/fr/user-manual.pdf` se contrôle aplati
   (`pdftotext … - | tr '\n' ' ' | tr -s ' '`), sur des fragments courts, aux **deux** apostrophes (`'` et `’`) ;
   jamais de `grep -oE` à quantificateur `{0,N}` sur le PDF : `grep -oF … | wc -l` sous `timeout 60`. La voie
   « rattacher la transaction à l'écriture de ce règlement » suppose la v017-3a (C-v017-3-35) : la juger comme telle.
7. **Les textes publics** : `docs/api-external.md` (paragraphe des lignes au centime, changement de contrat, ordre des
   refus de la complétion, `:602`), `CHANGELOG.md` (en-tête provisoire `[0.17.0]`, lien #687, aucun mot-clé de
   fermeture), les trois doc-comments de #687 complétés et non effacés.
8. **Les tests** : chaque test prouve-t-il ce qu'il nomme (mutations M1 à M17 de la fiche et leur sort au Dev Agent
   Record) ; un test qui passerait à vide ; la ligne de banque au centime de l'AC-11.

Lentille **B** : le diff seul, sans la fiche, défauts de code. Lentille **E** : chemins et bornes (échelles, signes,
zéros, virgule, très grands nombres, rang hors bornes, facture annulée, payée par lot, restauration). Lentille **A** :
chaque AC et chaque décision de la fiche contre le code, le manuel et le PDF.
